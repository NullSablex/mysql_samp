# Migration examples: before and after

Real Pawn snippets showing the **before** (R41-4) and the **after** (mysql_samp). For the explained guide and the full reference of differences, see [migration.md](migration.md) and [migration-changes.md](migration-changes.md).

---

## 1. Basic connection

### R41-4

```pawn
new MySQL:g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "password", "samp");
    if (g_mysql == MYSQL_INVALID_HANDLE)
    {
        printf("MySQL connection failed.");
        return 0;
    }
    return 1;
}
```

### mysql_samp

```pawn
new g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "password", "samp");
    if (g_mysql == 0)
    {
        printf("MySQL connection failed.");
        return 0;
    }
    return 1;
}
```

> **Changes:** no `MySQL:` tag, no `MYSQL_INVALID_HANDLE` (use plain `0`).

---

## 2. Connect with a custom port

### R41-4

```pawn
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:g_mysql = mysql_connect("127.0.0.1", "root", "pass", "samp", opt);
```

### mysql_samp

```pawn
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new g_mysql = mysql_connect("127.0.0.1", "root", "pass", "samp", opt);
```

> **Changes:** no tags, `mysql_init_options` → `mysql_options_new`, `mysql_set_option` → `mysql_options_set_int`, `SERVER_PORT` → `MYSQL_OPT_PORT`.

---

## 3. Check whether a player exists

### R41-4

```pawn
public OnPlayerConnect(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT * FROM players WHERE name = '%e'", name);
    mysql_tquery(g_mysql, query, "OnCheckAccount", "i", playerid);
    return 1;
}

forward OnCheckAccount(playerid);
public OnCheckAccount(playerid)
{
    if (cache_num_rows() > 0)
    {
        ShowPlayerDialog(playerid, DIALOG_LOGIN, ...);
    }
    else
    {
        ShowPlayerDialog(playerid, DIALOG_REGISTER, ...);
    }
    return 1;
}
```

### mysql_samp

```pawn
public OnPlayerConnect(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT * FROM players WHERE name = '%s'", name);
    mysql_query(g_mysql, query, "OnCheckAccount", "d", playerid);
    return 1;
}

forward OnCheckAccount(playerid);
public OnCheckAccount(playerid)
{
    if (cache_get_row_count() > 0)
    {
        ShowPlayerDialog(playerid, DIALOG_LOGIN, ...);
    }
    else
    {
        ShowPlayerDialog(playerid, DIALOG_REGISTER, ...);
    }
    return 1;
}
```

> **Changes:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `"i"` → `"d"` (optional), `cache_num_rows()` → `cache_get_row_count()`.

---

## 4. Load player data

### R41-4

```pawn
forward OnPlayerLogin(playerid);
public OnPlayerLogin(playerid)
{
    if (cache_num_rows() > 0)
    {
        cache_get_value_name(0, "hash", Player[playerid][Hash], 65);

        cache_get_value_name_int(0, "score", Player[playerid][Score]);
        cache_get_value_name_int(0, "money", Player[playerid][Money]);
        cache_get_value_name_float(0, "pos_x", Player[playerid][PosX]);
        cache_get_value_name_float(0, "pos_y", Player[playerid][PosY]);
        cache_get_value_name_float(0, "pos_z", Player[playerid][PosZ]);
        cache_get_value_name_int(0, "skin", Player[playerid][Skin]);
    }
    return 1;
}
```

### mysql_samp

```pawn
forward OnPlayerLogin(playerid);
public OnPlayerLogin(playerid)
{
    if (cache_get_row_count() > 0)
    {
        cache_get_value_name(0, "hash", Player[playerid][Hash]);

        Player[playerid][Score] = cache_get_value_name_int(0, "score");
        Player[playerid][Money] = cache_get_value_name_int(0, "money");
        Player[playerid][PosX]  = cache_get_value_name_float(0, "pos_x");
        Player[playerid][PosY]  = cache_get_value_name_float(0, "pos_y");
        Player[playerid][PosZ]  = cache_get_value_name_float(0, "pos_z");
        Player[playerid][Skin]  = cache_get_value_name_int(0, "skin");
    }
    return 1;
}
```

> **Changes:** `cache_num_rows()` → `cache_get_row_count()`. `cache_get_value_name_int` / `_float` switched from by-ref (3 params) to return value (2 params). `cache_get_value_name` for strings keeps the same shape (using the default `sizeof` argument).

---

## 5. Register — insert a new player

### R41-4

```pawn
public OnDialogResponse(playerid, dialogid, response, listitem, inputtext[])
{
    if (dialogid == DIALOG_REGISTER && response)
    {
        new query[256], name[MAX_PLAYER_NAME], hash[65];
        GetPlayerName(playerid, name, sizeof(name));
        SHA256_PassHash(inputtext, SALT, hash, sizeof(hash));

        mysql_format(g_mysql, query, sizeof(query),
            "INSERT INTO players (name, hash) VALUES ('%e', '%e')",
            name, hash);
        mysql_tquery(g_mysql, query, "OnPlayerRegister", "i", playerid);
    }
    return 1;
}

forward OnPlayerRegister(playerid);
public OnPlayerRegister(playerid)
{
    Player[playerid][ID] = cache_insert_id();
    SendClientMessage(playerid, -1, "Account created.");
    return 1;
}
```

### mysql_samp

```pawn
public OnDialogResponse(playerid, dialogid, response, listitem, inputtext[])
{
    if (dialogid == DIALOG_REGISTER && response)
    {
        new query[256], name[MAX_PLAYER_NAME], hash[65];
        GetPlayerName(playerid, name, sizeof(name));
        SHA256_PassHash(inputtext, SALT, hash, sizeof(hash));

        mysql_format(g_mysql, query, sizeof(query),
            "INSERT INTO players (name, hash) VALUES ('%s', '%s')",
            name, hash);
        mysql_query(g_mysql, query, "OnPlayerRegister", "d", playerid);
    }
    return 1;
}

forward OnPlayerRegister(playerid);
public OnPlayerRegister(playerid)
{
    Player[playerid][ID] = cache_insert_id();
    SendClientMessage(playerid, -1, "Account created.");
    return 1;
}
```

> **Changes:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `"i"` → `"d"` (optional).

---

## 6. Save player data

### R41-4

```pawn
stock SavePlayer(playerid)
{
    new query[512];
    mysql_format(g_mysql, query, sizeof(query),
        "UPDATE players SET score = %d, money = %d, pos_x = %f, pos_y = %f, pos_z = %f WHERE id = %d",
        GetPlayerScore(playerid),
        GetPlayerMoney(playerid),
        Player[playerid][PosX],
        Player[playerid][PosY],
        Player[playerid][PosZ],
        Player[playerid][ID]);
    mysql_tquery(g_mysql, query);
    return 1;
}
```

### mysql_samp

```pawn
stock SavePlayer(playerid)
{
    new query[512];
    mysql_format(g_mysql, query, sizeof(query),
        "UPDATE players SET score = %d, money = %d, pos_x = %f, pos_y = %f, pos_z = %f WHERE id = %d",
        GetPlayerScore(playerid),
        GetPlayerMoney(playerid),
        Player[playerid][PosX],
        Player[playerid][PosY],
        Player[playerid][PosZ],
        Player[playerid][ID]);
    mysql_query(g_mysql, query);
    return 1;
}
```

> **Minimal change:** `mysql_tquery` → `mysql_query`. No callback, no format — fire-and-forget.

---

## 7. VIP check — query with callback and parameters

### R41-4

```pawn
stock CheckVIP(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT vip_level, vip_expires FROM players WHERE name = '%e'", name);
    mysql_tquery(g_mysql, query, "OnVIPCheck", "i", playerid);
}

forward OnVIPCheck(playerid);
public OnVIPCheck(playerid)
{
    if (cache_num_rows() > 0)
    {
        new level, expires[20];
        cache_get_value_name_int(0, "vip_level", level);
        cache_get_value_name(0, "vip_expires", expires, sizeof(expires));

        Player[playerid][VIPLevel] = level;
    }
    return 1;
}
```

### mysql_samp

```pawn
stock CheckVIP(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT vip_level, vip_expires FROM players WHERE name = '%s'", name);
    mysql_query(g_mysql, query, "OnVIPCheck", "d", playerid);
}

forward OnVIPCheck(playerid);
public OnVIPCheck(playerid)
{
    if (cache_get_row_count() > 0)
    {
        new expires[20];
        Player[playerid][VIPLevel] = cache_get_value_name_int(0, "vip_level");
        cache_get_value_name(0, "vip_expires", expires);
    }
    return 1;
}
```

> **Changes:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `cache_num_rows` → `cache_get_row_count`, `cache_get_value_name_int` switched to return value, `cache_get_value_name` works with the default `sizeof`.

---

## 8. Ban system

### R41-4

```pawn
stock BanPlayer(playerid, adminid, const reason[])
{
    new query[256], name[MAX_PLAYER_NAME], admin[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));
    GetPlayerName(adminid,  admin, sizeof(admin));

    mysql_format(g_mysql, query, sizeof(query),
        "INSERT INTO bans (name, admin, reason, ip) VALUES ('%e', '%e', '%e', '%e')",
        name, admin, reason, Player[playerid][IP]);
    mysql_tquery(g_mysql, query);
    Kick(playerid);
}

stock CheckBan(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT reason FROM bans WHERE name = '%e'", name);
    mysql_tquery(g_mysql, query, "OnBanCheck", "i", playerid);
}

forward OnBanCheck(playerid);
public OnBanCheck(playerid)
{
    if (cache_num_rows() > 0)
    {
        new reason[128];
        cache_get_value_name(0, "reason", reason, sizeof(reason));
        SendClientMessage(playerid, -1, reason);
        Kick(playerid);
    }
    return 1;
}
```

### mysql_samp

```pawn
stock BanPlayer(playerid, adminid, const reason[])
{
    new query[256], name[MAX_PLAYER_NAME], admin[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));
    GetPlayerName(adminid,  admin, sizeof(admin));

    mysql_format(g_mysql, query, sizeof(query),
        "INSERT INTO bans (name, admin, reason, ip) VALUES ('%s', '%s', '%s', '%s')",
        name, admin, reason, Player[playerid][IP]);
    mysql_query(g_mysql, query);
    Kick(playerid);
}

stock CheckBan(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(g_mysql, query, sizeof(query),
        "SELECT reason FROM bans WHERE name = '%s'", name);
    mysql_query(g_mysql, query, "OnBanCheck", "d", playerid);
}

forward OnBanCheck(playerid);
public OnBanCheck(playerid)
{
    if (cache_get_row_count() > 0)
    {
        new reason[128];
        cache_get_value_name(0, "reason", reason);
        SendClientMessage(playerid, -1, reason);
        Kick(playerid);
    }
    return 1;
}
```

---

## 9. Load vehicles from the database

### R41-4

```pawn
stock LoadVehicles()
{
    mysql_tquery(g_mysql, "SELECT * FROM vehicles", "OnVehiclesLoaded");
}

forward OnVehiclesLoaded();
public OnVehiclesLoaded()
{
    new rows = cache_num_rows();
    for (new i = 0; i < rows; i++)
    {
        new model, Float:x, Float:y, Float:z, Float:a, color1, color2;

        cache_get_value_name_int(i,   "model",  model);
        cache_get_value_name_float(i, "pos_x",  x);
        cache_get_value_name_float(i, "pos_y",  y);
        cache_get_value_name_float(i, "pos_z",  z);
        cache_get_value_name_float(i, "angle",  a);
        cache_get_value_name_int(i,   "color1", color1);
        cache_get_value_name_int(i,   "color2", color2);

        CreateVehicle(model, x, y, z, a, color1, color2, -1);
    }
    printf("vehicles loaded: %d", rows);
    return 1;
}
```

### mysql_samp

```pawn
stock LoadVehicles()
{
    mysql_query(g_mysql, "SELECT * FROM vehicles", "OnVehiclesLoaded");
}

forward OnVehiclesLoaded();
public OnVehiclesLoaded()
{
    new rows = cache_get_row_count();
    for (new i = 0; i < rows; i++)
    {
        new model        = cache_get_value_name_int(i,   "model");
        new Float:x      = cache_get_value_name_float(i, "pos_x");
        new Float:y      = cache_get_value_name_float(i, "pos_y");
        new Float:z      = cache_get_value_name_float(i, "pos_z");
        new Float:a      = cache_get_value_name_float(i, "angle");
        new color1       = cache_get_value_name_int(i,   "color1");
        new color2       = cache_get_value_name_int(i,   "color2");

        CreateVehicle(model, x, y, z, a, color1, color2, -1);
    }
    printf("vehicles loaded: %d", rows);
    return 1;
}
```

> **Changes:** `mysql_tquery` → `mysql_query`, `cache_num_rows` → `cache_get_row_count`, by-ref → return value for `_int` and `_float`.

---

## 10. ORM — full player flow

### R41-4

```pawn
enum pInfo
{
    ORM:ORM_ID,
    ID,
    Name[MAX_PLAYER_NAME],
    Hash[65],
    Score,
    Money,
    Float:PosX,
    Float:PosY,
    Float:PosZ,
};
new Player[MAX_PLAYERS][pInfo];

stock CreateORM(playerid)
{
    new ORM:orm = orm_create("players", g_mysql);
    Player[playerid][ORM_ID] = orm;

    orm_addvar_int(orm,    Player[playerid][ID],    "id");
    orm_addvar_string(orm, Player[playerid][Name],  MAX_PLAYER_NAME, "name");
    orm_addvar_string(orm, Player[playerid][Hash],  65, "hash");
    orm_addvar_int(orm,    Player[playerid][Score], "score");
    orm_addvar_int(orm,    Player[playerid][Money], "money");
    orm_addvar_float(orm,  Player[playerid][PosX],  "pos_x");
    orm_addvar_float(orm,  Player[playerid][PosY],  "pos_y");
    orm_addvar_float(orm,  Player[playerid][PosZ],  "pos_z");

    orm_setkey(orm, "id");
    return _:orm;
}

orm_select(Player[playerid][ORM_ID], "OnPlayerLoad", "i", playerid);

forward OnPlayerLoad(playerid);
public OnPlayerLoad(playerid)
{
    orm_apply_cache(Player[playerid][ORM_ID], 0);
    SetPlayerScore(playerid, Player[playerid][Score]);
    GivePlayerMoney(playerid, Player[playerid][Money]);
}

orm_save(Player[playerid][ORM_ID]);
```

### mysql_samp

```pawn
enum pInfo
{
    ORM_ID,
    ID,
    Name[MAX_PLAYER_NAME],
    Hash[65],
    Score,
    Money,
    Float:PosX,
    Float:PosY,
    Float:PosZ,
};
new Player[MAX_PLAYERS][pInfo];

stock CreateORM(playerid)
{
    new orm = orm_create("players", g_mysql);
    Player[playerid][ORM_ID] = orm;

    orm_addvar_int(orm,    Player[playerid][ID],    "id");
    orm_addvar_string(orm, Player[playerid][Name],  MAX_PLAYER_NAME, "name");
    orm_addvar_string(orm, Player[playerid][Hash],  65, "hash");
    orm_addvar_int(orm,    Player[playerid][Score], "score");
    orm_addvar_int(orm,    Player[playerid][Money], "money");
    orm_addvar_float(orm,  Player[playerid][PosX],  "pos_x");
    orm_addvar_float(orm,  Player[playerid][PosY],  "pos_y");
    orm_addvar_float(orm,  Player[playerid][PosZ],  "pos_z");

    orm_setkey(orm, "id");
    return orm;
}

orm_select(Player[playerid][ORM_ID], "OnPlayerLoad", "d", playerid);

forward OnPlayerLoad(playerid);
public OnPlayerLoad(playerid)
{
    orm_apply_cache(Player[playerid][ORM_ID], 0);
    SetPlayerScore(playerid, Player[playerid][Score]);
    GivePlayerMoney(playerid, Player[playerid][Money]);
}

orm_save(Player[playerid][ORM_ID]);
```

> **Changes:** no `ORM:` tag (plain `int`), `"i"` → `"d"` (optional). Everything else is identical.

---

## 11. Saved cache — store and reuse

### R41-4

```pawn
forward OnQueryA(playerid);
public OnQueryA(playerid)
{
    new Cache:cache = cache_save();

    mysql_tquery(g_mysql, "SELECT ...", "OnQueryB", "ii", playerid, _:cache);
    return 1;
}

forward OnQueryB(playerid, cache_id);
public OnQueryB(playerid, cache_id)
{
    new rows = cache_num_rows();
    // ...

    cache_set_active(Cache:cache_id);
    new rowsA = cache_num_rows();
    // ...
    cache_unset_active();

    cache_delete(Cache:cache_id);
    return 1;
}
```

### mysql_samp

```pawn
forward OnQueryA(playerid);
public OnQueryA(playerid)
{
    new cache = cache_save();

    mysql_query(g_mysql, "SELECT ...", "OnQueryB", "dd", playerid, cache);
    return 1;
}

forward OnQueryB(playerid, cache_id);
public OnQueryB(playerid, cache_id)
{
    new rows = cache_get_row_count();
    // ...

    cache_set_active(cache_id);
    new rowsA = cache_get_row_count();
    // ...
    cache_unset_active();

    cache_delete(cache_id);
    return 1;
}
```

> **Changes:** no `Cache:` tag, `mysql_tquery` → `mysql_query`, `"ii"` → `"dd"` (optional — `"ii"` works too), `cache_num_rows` → `cache_get_row_count`.

---

## 12. Error handling

### R41-4

```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], MySQL:handle);
public OnQueryError(errorid, const error[], const callback[], const query[], MySQL:handle)
{
    printf("[MySQL] error %d on query: %s", errorid, error);
    printf("[MySQL] callback: %s | query: %s", callback, query);
    return 1;
}
```

### mysql_samp

```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[MySQL] error %d on query: %s", errorid, error);
    printf("[MySQL] callback: %s | query: %s", callback, query);

    // bonus: fetch the connection-scoped error text
    new err[256];
    mysql_error(connId, err);
    printf("[MySQL] detail: %s", err);
    return 1;
}
```

> **Change:** no `MySQL:` tag on the last parameter. As a bonus, `mysql_error(connId, dest)` gives you the per-connection error text.

---

## 13. Escape a string

### R41-4

```pawn
new escaped[128];
mysql_escape_string(input, escaped, sizeof(escaped), g_mysql);
```

### mysql_samp

```pawn
new escaped[128];
mysql_escape_string(input, escaped, sizeof(escaped), g_mysql);
```

> **Changes:** the handle stays (as `connId`) and should be passed — it selects the escaping rules for the server's `sql_mode`. The charset is always UTF-8; the plugin forces `SET NAMES utf8mb4`.

---

## 14. NULL check

### R41-4

```pawn
new bool:is_null;
cache_is_value_name_null(0, "email", is_null);
if (is_null)
{
    // column is NULL
}
```

### mysql_samp

```pawn
if (cache_is_value_name_null(0, "email"))
{
    // column is NULL
}
```

> **Change:** return value instead of by-ref. Cleaner.

---

## Quick find-and-replace checklist

For a mechanical first pass, search and replace in your gamemode:

| Find | Replace with |
|---|---|
| `#include <a_mysql>` | `#include <mysql_samp>` |
| `mysql_tquery(` | `mysql_query(` |
| `cache_num_rows(` | `cache_get_row_count(` |
| `cache_num_fields(` | `cache_get_field_count(` |
| `mysql_init_options(` | `mysql_options_new(` |
| `mysql_stat(` | `mysql_status(` |
| `new MySQL:` | `new ` |
| `new Cache:` | `new ` |
| `new ORM:` | `new ` |
| `new MySQLOpt:` | `new ` |
| `Cache:` (in calls) | *(remove)* |
| `ORM:` (in calls) | *(remove)* |
| `MySQL:` (in calls) | *(remove)* |
| `MySQLOpt:` (in calls) | *(remove)* |
| `MYSQL_INVALID_HANDLE` | `0` |
| `MYSQL_DEFAULT_HANDLE` | *(remove — use the variable directly)* |

> **Needs manual review:**
> - `cache_get_value_*_int` / `_float`: change from 3-arg by-ref to 2-arg return value.
> - `cache_is_value_*_null`: change from 3-arg by-ref to 2-arg return value.
> - `cache_get_row_count` / `cache_get_field_count`: change from by-ref to return value.
> - `mysql_escape_string`: keep the connection — it selects the escaping rules for the server's `sql_mode`. Prefer prepared statements (`mysql_stmt_*`) for player input.
> - `mysql_error`: reverse the order — `mysql_error(connId, dest)` instead of `mysql_error(dest, max_len, handle)`.
> - `mysql_set_option` → `mysql_options_set_int` / `mysql_options_set_str`.
> - `%s` → `%r` where the old `%s` was intentionally raw. `%e` → `%s` (or keep `%e`).
