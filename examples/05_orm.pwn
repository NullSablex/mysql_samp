// 05_orm.pwn — minimal ORM: bind Pawn vars to columns, save / load by key.
//
// Workflow:
//   1. orm_create(table, conn) — returns an orm_id tied to that table.
//   2. orm_addvar_* — bind a Pawn variable to a column. Reads write into the
//      var; writes pull the var's current value.
//   3. orm_setkey(orm_id, "id") — declare the primary key. orm_save() decides
//      INSERT vs UPDATE based on whether the key var is zero/empty.
//   4. orm_select / orm_save / orm_delete / orm_insert / orm_update — async,
//      each with an optional callback.
//
// Schema assumed for the example:
//   CREATE TABLE players (
//     id    INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
//     name  VARCHAR(24) NOT NULL UNIQUE,
//     money INT NOT NULL DEFAULT 0,
//     score INT NOT NULL DEFAULT 0
//   );

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

// One ORM instance per player slot.
new gPlayerOrm[MAX_PLAYERS];
new gPlayerDbId[MAX_PLAYERS];
new gPlayerName[MAX_PLAYERS][MAX_PLAYER_NAME];
new gPlayerMoney[MAX_PLAYERS];
new gPlayerScore[MAX_PLAYERS];

public OnGameModeInit()
{
    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE);
    return 1;
}

public OnPlayerConnect(playerid)
{
    GetPlayerName(playerid, gPlayerName[playerid], MAX_PLAYER_NAME);
    gPlayerDbId[playerid]  = 0;
    gPlayerMoney[playerid] = 0;
    gPlayerScore[playerid] = 0;

    new orm = orm_create("players", g_MysqlConn);
    orm_addvar_int   (orm, gPlayerDbId[playerid],       "id");
    orm_addvar_string(orm, gPlayerName[playerid], MAX_PLAYER_NAME, "name");
    orm_addvar_int   (orm, gPlayerMoney[playerid],      "money");
    orm_addvar_int   (orm, gPlayerScore[playerid],      "score");
    orm_setkey(orm, "id");

    gPlayerOrm[playerid] = orm;

    // First load attempt is by name (key is zero, so SELECT uses bound vars).
    // We use a raw query for the WHERE-by-name lookup since orm_select keys on
    // the primary key.
    new query[256];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "SELECT id FROM players WHERE name = '%e' LIMIT 1",
                 gPlayerName[playerid]);
    mysql_query(g_MysqlConn, query, "OnPlayerLookup", "d", playerid);
    return 1;
}

forward OnPlayerLookup(playerid);
public  OnPlayerLookup(playerid)
{
    if (cache_get_row_count() == 0)
    {
        // New player: orm_save with id=0 → INSERT.
        orm_save(gPlayerOrm[playerid], "OnPlayerInserted", "d", playerid);
        return 1;
    }

    // Existing player: store the id and pull the rest via orm_select.
    gPlayerDbId[playerid] = cache_get_value_name_int(0, "id");
    orm_select(gPlayerOrm[playerid], "OnPlayerLoaded", "d", playerid);
    return 1;
}

forward OnPlayerLoaded(playerid);
public  OnPlayerLoaded(playerid)
{
    if (orm_errno(gPlayerOrm[playerid]) != ORM_OK) return 1;
    GivePlayerMoney(playerid, gPlayerMoney[playerid]);
    SetPlayerScore (playerid, gPlayerScore[playerid]);
    return 1;
}

forward OnPlayerInserted(playerid);
public  OnPlayerInserted(playerid)
{
    gPlayerDbId[playerid] = cache_insert_id();
    printf("[orm] created player %d (db_id=%d)", playerid, gPlayerDbId[playerid]);
    return 1;
}

public OnPlayerDisconnect(playerid, reason)
{
    gPlayerMoney[playerid] = GetPlayerMoney(playerid);
    gPlayerScore[playerid] = GetPlayerScore(playerid);

    if (gPlayerOrm[playerid] != 0 && gPlayerDbId[playerid] != 0)
    {
        // Key is set → orm_save performs an UPDATE.
        orm_save(gPlayerOrm[playerid]);
        orm_destroy(gPlayerOrm[playerid]);
        gPlayerOrm[playerid] = 0;
    }
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
