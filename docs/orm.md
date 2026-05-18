# ORM

The ORM maps Pawn variables to MySQL columns. With it, SELECT / INSERT / UPDATE / DELETE / save are one-liners — the plugin generates the SQL from the bound variables.

## Concept

1. Create an ORM instance attached to a table and a connection.
2. Bind Pawn variables to columns (`orm_addvar_*`).
3. Declare the primary key column (`orm_setkey`).
4. Run CRUD operations — every call reads the current values from the bound variables, generates the SQL, and runs the query through the standard threaded pipeline.

The five threaded CRUD natives (`orm_select`, `orm_update`, `orm_insert`, `orm_delete`, `orm_save`) accept an optional callback with the **same format string convention as `mysql_query`** (`d`/`i`, `f`, `s`).

ORM instances are owned per-AMX. When the AMX that created the instance unloads, the plugin destroys every ORM bound to it automatically — `Plugin::on_amx_unload` calls `OrmManager::destroy_by_amx`. You do not need to clean up on script unload.

## Lifecycle

### orm_create

```pawn
native orm_create(const table[], connId);
```

Creates an ORM bound to `table` on `connId`. Returns the ORM id (`>= 1`) or `0` if `connId` does not exist.

### orm_destroy

```pawn
native bool:orm_destroy(orm_id);
```

Returns `true` if the ORM existed and was removed, `false` otherwise.

```pawn
new g_orm;

public OnGameModeInit()
{
    g_orm = orm_create("players", g_mysql);
    // ... bind variables, set key ...
}

public OnGameModeExit()
{
    orm_destroy(g_orm);
}
```

## Variable bindings

```pawn
native bool:orm_addvar_int(orm_id, &var, const column_name[]);
native bool:orm_addvar_float(orm_id, &Float:var, const column_name[]);
native bool:orm_addvar_string(orm_id, var[], var_max_len, const column_name[]);
native bool:orm_delvar(orm_id, const column_name[]);
native bool:orm_clear_vars(orm_id);
native bool:orm_setkey(orm_id, const column_name[]);
```

- `orm_addvar_int` / `orm_addvar_float` take Pawn variables **by reference** — the ORM stores the AMX address, not the value, so changes from Pawn are picked up on the next query build.
- `orm_addvar_string` requires `1 <= var_max_len <= 4096`. Values outside that range are rejected (`false`). The 4096 cap protects the AMX heap against oversized writes when `orm_apply_cache` copies a column into the Pawn buffer.
- `orm_delvar(column_name)` removes the binding whose column name matches exactly (case-sensitive).
- `orm_clear_vars(orm_id)` drops every binding.
- `orm_setkey(column_name)` declares which bound column is the primary key. Required for `orm_select`, `orm_update`, `orm_delete`. `orm_insert` works without a key.

### Setup pattern

```pawn
enum PlayerData
{
    pId,
    pName[MAX_PLAYER_NAME],
    Float:pScore,
    pLevel
}

new g_player_data[MAX_PLAYERS][PlayerData];
new g_player_orm[MAX_PLAYERS];

stock SetupPlayerORM(playerid)
{
    new oid = orm_create("players", g_mysql);
    g_player_orm[playerid] = oid;

    orm_addvar_int(oid,    g_player_data[playerid][pId],      "id");
    orm_addvar_string(oid, g_player_data[playerid][pName],    MAX_PLAYER_NAME, "name");
    orm_addvar_float(oid,  g_player_data[playerid][pScore],   "score");
    orm_addvar_int(oid,    g_player_data[playerid][pLevel],   "level");

    orm_setkey(oid, "id");
}
```

## CRUD operations

Every CRUD native shares the same signature:

```pawn
native bool:orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_save(orm_id,   const callback[] = "", const format[] = "", {Float,_}:...);
```

All five route through `mysql_query` (FIFO ordering, non-blocking). Each returns `false` synchronously when:

- `orm_id` is unknown (`MYSQL_ERROR_INVALID_ORM`),
- the query cannot be built (`MYSQL_ERROR_ORM_KEY_NOT_SET` for select/update/delete, `MYSQL_ERROR_INVALID_ORM` for insert/save),
- the underlying connection has gone away.

### orm_select

Generated SQL: `SELECT col1, col2, … FROM \`table\` WHERE \`key\` = <current key value>`.

```pawn
g_player_data[playerid][pId] = db_id;
orm_select(g_player_orm[playerid], "OnPlayerDataLoaded", "d", playerid);

forward OnPlayerDataLoaded(playerid);
public OnPlayerDataLoaded(playerid)
{
    // copy cache row 0 into the bound variables
    orm_apply_cache(g_player_orm[playerid]);

    if (orm_errno(g_player_orm[playerid]) == ORM_NO_DATA)
    {
        printf("no row for player %d", playerid);
        return;
    }

    printf("loaded %s (level %d)",
        g_player_data[playerid][pName],
        g_player_data[playerid][pLevel]);
}
```

### orm_insert

Generated SQL: `INSERT INTO \`table\` (col1, col2, …) VALUES (val1, val2, …)`.

```pawn
g_player_data[playerid][pName]  = "NewPlayer";
g_player_data[playerid][pLevel] = 1;
g_player_data[playerid][pScore] = 0.0;

orm_insert(g_player_orm[playerid], "OnPlayerInserted", "d", playerid);

forward OnPlayerInserted(playerid);
public OnPlayerInserted(playerid)
{
    g_player_data[playerid][pId] = cache_insert_id();
    printf("inserted with id %d", g_player_data[playerid][pId]);
}
```

### orm_update

Generated SQL: `UPDATE \`table\` SET col1=val1, col2=val2, … WHERE \`key\` = <current key value>`.

```pawn
g_player_data[playerid][pLevel] = 10;
g_player_data[playerid][pScore] = 1500.0;

orm_update(g_player_orm[playerid]);
```

### orm_delete

Generated SQL: `DELETE FROM \`table\` WHERE \`key\` = <current key value>`.

```pawn
orm_delete(g_player_orm[playerid], "OnPlayerDeleted", "d", playerid);

forward OnPlayerDeleted(playerid);
public OnPlayerDeleted(playerid)
{
    printf("player %d removed", playerid);
}
```

### orm_save

Decides between INSERT and UPDATE by looking at the current value of the key column:

- Int key equals `0` → INSERT.
- Float key equals `0.0` → INSERT.
- String key is empty (`""`) → INSERT.
- Anything else → UPDATE.

```pawn
// First save after creation does an INSERT (id is 0), later saves do UPDATE.
orm_save(g_player_orm[playerid], "OnPlayerSaved", "d", playerid);
```

## orm_apply_cache

```pawn
native bool:orm_apply_cache(orm_id, row = 0);
```

Writes one row of the active cache into the bound variables. Must be called from a query callback, while the cache is active. Returns `true` on success.

Failure modes:

| Condition | Return | `orm_errno` |
|---|---|---|
| No cache currently active | `false` | unchanged (sets global plugin error to `MYSQL_ERROR_NO_CACHE_ACTIVE`) |
| `orm_id` is unknown | `false` | unchanged (sets global plugin error to `MYSQL_ERROR_INVALID_ORM`) |
| `row` is negative or `>= cache_get_row_count()` | `false` | `ORM_NO_DATA` |

When a row index is valid, the plugin walks each binding and copies the matching column into the Pawn variable. Bindings whose column is missing from the cache are silently skipped (the Pawn variable keeps its current value).

## orm_errno

```pawn
native orm_errno(orm_id);
```

Returns the ORM error code for the last operation on the instance, or `-1` if `orm_id` is unknown.

| Code | Constant | Meaning |
|---|---|---|
| 0 | `ORM_OK` | The last `orm_apply_cache` succeeded |
| 1 | `ORM_NO_DATA` | The last `orm_apply_cache` failed because the requested row does not exist |

This errno is **only updated by `orm_apply_cache`**. The threaded CRUD natives (`orm_select`/insert/update/delete/save) surface their failures synchronously (return `false`) and through the global `mysql_errno(0)` / `OnQueryError`, not through `orm_errno`.

## Generated SQL: safety

- String columns are escaped through the same `escape_string` used by `mysql_format %s`.
- Table and column names are sanitized through `escape_identifier`, which strips backticks. The output is then wrapped in backticks.
- Numeric columns are formatted directly (`{}` for ints, `{}` for floats — no locale-aware formatting).

The result is safe against SQL injection through the bound variables. **Do not** insert hostile column names through `orm_setkey` or `orm_addvar_*`; the plugin removes backticks but does not run a full identifier validator.

## End-to-end example

```pawn
#include <a_samp>
#include <mysql_samp>

#define MAX_PLAYER_NAME 24

enum pInfo
{
    pDBId,
    pName[MAX_PLAYER_NAME],
    pLevel,
    Float:pMoney
}

new PlayerInfo[MAX_PLAYERS][pInfo];
new PlayerORM[MAX_PLAYERS];
new g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "", "samp_server");
    if (mysql_errno() != MYSQL_OK)
    {
        printf("[MySQL] connection failed");
        return 1;
    }
    return 1;
}

public OnPlayerConnect(playerid)
{
    new oid = orm_create("players", g_mysql);
    PlayerORM[playerid] = oid;

    orm_addvar_int(oid,    PlayerInfo[playerid][pDBId],  "id");
    orm_addvar_string(oid, PlayerInfo[playerid][pName],  MAX_PLAYER_NAME, "name");
    orm_addvar_int(oid,    PlayerInfo[playerid][pLevel], "level");
    orm_addvar_float(oid,  PlayerInfo[playerid][pMoney], "money");
    orm_setkey(oid, "id");

    GetPlayerName(playerid, PlayerInfo[playerid][pName], MAX_PLAYER_NAME);

    new query[128];
    mysql_format(g_mysql, query, sizeof(query),
        "SELECT * FROM players WHERE name = '%s' LIMIT 1",
        PlayerInfo[playerid][pName]);
    mysql_query(g_mysql, query, "OnPlayerLookup", "d", playerid);
    return 1;
}

forward OnPlayerLookup(playerid);
public OnPlayerLookup(playerid)
{
    if (cache_get_row_count() > 0)
    {
        orm_apply_cache(PlayerORM[playerid]);
        printf("loaded %s (id=%d, level=%d)",
            PlayerInfo[playerid][pName],
            PlayerInfo[playerid][pDBId],
            PlayerInfo[playerid][pLevel]);
    }
    else
    {
        // brand-new player
        PlayerInfo[playerid][pLevel] = 1;
        PlayerInfo[playerid][pMoney] = 0.0;
        orm_insert(PlayerORM[playerid], "OnPlayerCreated", "d", playerid);
    }
}

forward OnPlayerCreated(playerid);
public OnPlayerCreated(playerid)
{
    PlayerInfo[playerid][pDBId] = cache_insert_id();
    printf("new player saved with id %d", PlayerInfo[playerid][pDBId]);
}

public OnPlayerDisconnect(playerid, reason)
{
    orm_save(PlayerORM[playerid]);
    orm_destroy(PlayerORM[playerid]);
    return 1;
}
```
