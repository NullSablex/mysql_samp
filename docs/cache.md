# Cache

The cache holds the result of a single query so that the `cache_*` natives can read it. There are two ways a cache becomes "active":

1. **Automatic stack.** Before a query callback fires, the plugin pushes the result onto an internal stack. When the callback returns, the entry is popped. Inside the callback, every `cache_*` native reads from the top of the stack.
2. **Manual activation.** `cache_save()` clones the active entry into a persistent slot identified by an id. Later, `cache_set_active(id)` makes the saved entry override the stack top until `cache_unset_active()` is called.

```
mysql_query → worker thread → result → push → callback() → pop
```

Outside a callback (and without `cache_set_active`), the cache natives return their "no active cache" sentinels:

| Native shape | Sentinel |
|---|---|
| `cache_get_*` returning `int` | `-1` |
| `cache_get_*_int` | `0` |
| `cache_get_*_float` | `0.0` |
| `cache_get_*` writing to `dest` | `false`, `dest` untouched |
| `cache_is_value_*_null` | `true` (treat absence as NULL) |

## Reading rows

### Dimensions

```pawn
native cache_get_row_count();
native cache_get_field_count();
```

Both return `-1` when no cache is active.

```pawn
forward OnQueryDone();
public OnQueryDone()
{
    new rows   = cache_get_row_count();
    new fields = cache_get_field_count();
    printf("result: %d rows x %d fields", rows, fields);
}
```

### By column index

```pawn
native bool:cache_get_value_index(row, col, dest[], max_len = sizeof(dest));
native cache_get_value_index_int(row, col);
native Float:cache_get_value_index_float(row, col);
```

`row` and `col` are zero-based and must be non-negative. Negative values are rejected without touching `dest`. Out-of-range or NULL cells return `false` / `0` / `0.0`.

```pawn
forward OnResult();
public OnResult()
{
    new rows = cache_get_row_count();
    for (new i = 0; i < rows; i++)
    {
        new id = cache_get_value_index_int(i, 0);
        new name[64];
        cache_get_value_index(i, 1, name);
        printf("id=%d name=%s", id, name);
    }
}
```

### By column name

```pawn
native bool:cache_get_value_name(row, const field_name[], dest[], max_len = sizeof(dest));
native cache_get_value_name_int(row, const field_name[]);
native Float:cache_get_value_name_float(row, const field_name[]);
```

The column lookup is **case-insensitive**: `"name"`, `"Name"` and `"NAME"` all match the same column.

```pawn
forward OnPlayerData();
public OnPlayerData()
{
    if (cache_get_row_count() <= 0) return;

    new name[MAX_PLAYER_NAME];
    cache_get_value_name(0, "username", name);

    new level = cache_get_value_name_int(0, "level");
    new Float:score = cache_get_value_name_float(0, "score");

    printf("%s — level %d — score %.2f", name, level, score);
}
```

### NULL checks

```pawn
native bool:cache_is_value_index_null(row, col);
native bool:cache_is_value_name_null(row, const field_name[]);
```

Returns `true` when the cell is SQL `NULL`, **also** when the row/column is out of bounds or no cache is active. The "no cache" → `true` mapping means downstream code that just wants to skip empty cells does not need an extra guard, but if you need to distinguish "missing row" from "NULL cell" check `cache_get_row_count()` first.

```pawn
if (!cache_is_value_name_null(0, "email"))
{
    new email[128];
    cache_get_value_name(0, "email", email);
    printf("email: %s", email);
}
```

## Metadata

### Columns

```pawn
native bool:cache_get_field_name(field_idx, dest[], max_len = sizeof(dest));
native cache_get_field_type(field_idx);
```

`cache_get_field_type` returns the raw MySQL column-type byte (`mysql::consts::ColumnType` as a `u8`). For example: `3` = `MYSQL_TYPE_LONG`, `253` = `MYSQL_TYPE_VAR_STRING`. Returns `-1` for out-of-range indices.

```pawn
new fields = cache_get_field_count();
for (new i = 0; i < fields; i++)
{
    new name[64];
    cache_get_field_name(i, name);
    printf("col %d: %s (type=%d)", i, name, cache_get_field_type(i));
}
```

### Write-result metadata

```pawn
native cache_affected_rows();      // -1 if no cache, otherwise rows affected
native cache_insert_id();          // -1 if no cache, otherwise last AUTO_INCREMENT
native cache_warning_count();      // -1 if no cache, otherwise warnings from the server
```

```pawn
forward OnPlayerInserted();
public OnPlayerInserted()
{
    printf("new player id: %d", cache_insert_id());
}

forward OnPlayersDeleted();
public OnPlayersDeleted()
{
    printf("rows deleted: %d", cache_affected_rows());
}
```

### Query metadata

```pawn
native cache_get_query_exec_time();                                  // milliseconds, -1 if no cache
native bool:cache_get_query_string(dest[], max_len = sizeof(dest));  // the SQL text actually executed
```

The exec time is computed by the worker thread, so it includes the round-trip to the MySQL server but not the dispatch delay back to the callback.

```pawn
printf("query took %d ms", cache_get_query_exec_time());

new query[512];
cache_get_query_string(query);
printf("query: %s", query);
```

## Persistent caches

By default the cache is dropped when the callback returns. To keep a result around, save it:

```pawn
native cache_save();                       // returns the saved id (>= 1) or 0 on failure
native bool:cache_delete(cache_id);
native bool:cache_set_active(cache_id);
native bool:cache_unset_active();
native bool:cache_is_any_active();
native bool:cache_is_valid(cache_id);
```

`cache_save()` returns `0` when no cache is active or when the saved-cache limit (1024) is reached.

```pawn
new g_saved;

forward OnDataLoaded();
public OnDataLoaded()
{
    g_saved = cache_save();
    printf("cache saved id=%d", g_saved);
}

stock UseSavedData()
{
    if (!cache_is_valid(g_saved))
    {
        printf("cache expired");
        return;
    }

    cache_set_active(g_saved);
    new rows = cache_get_row_count();
    printf("saved cache has %d rows", rows);
    cache_unset_active();
}

stock DropSavedData()
{
    cache_delete(g_saved);
    g_saved = 0;
}
```

`cache_delete` also clears the manual override if the deleted id is currently active. `cache_unset_active` returns `false` if there is no manual override to clear (the stack-top behavior continues normally).

## Limits

| Resource | Limit | What happens when hit |
|---|---|---|
| Saved caches | 1 024 | `cache_save()` returns `0`, warning logged |
| Rows per query result | 100 000 | Extra rows are drained from the protocol but discarded; one warning is logged |

The 100k-row limit prevents a single runaway `SELECT *` from blowing the server's memory.
