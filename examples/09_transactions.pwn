// 09_transactions.pwn — all-or-nothing batches.
//
// A transaction guarantees that a group of statements either all apply or none
// do. The classic case is moving money: without one, a crash or an error
// between the debit and the credit destroys or duplicates it.
//
// Steps are collected first and then run as a single unit on one connection:
//   START TRANSACTION -> every step -> COMMIT
// Any failing step rolls the whole batch back and fires OnQueryError.
//
// There is deliberately NO interactive begin/commit API. Holding a pooled
// connection between server ticks would leak it whenever a gamemode never
// reached the commit (an early return, a runtime error, a player disconnect).

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

public OnGameModeInit()
{
    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE);
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}

// --- Money transfer, with bound values ---------------------------------------
//
// mysql_transaction_add_stmt copies a prepared statement together with the
// values currently bound to it. This is the safe way to put player-controlled
// numbers or text inside a transaction.

TransferMoney(playerid, fromId, toId, amount)
{
    new tx = mysql_transaction_new(g_MysqlConn);
    if (tx == 0) return 0;

    new stmt = mysql_stmt_new(g_MysqlConn,
        "UPDATE accounts SET balance = balance - ? WHERE id = ? AND balance >= ?");

    if (stmt == 0)
    {
        mysql_transaction_destroy(tx);
        return 0;
    }

    // Debit
    mysql_stmt_bind_int(stmt, amount);
    mysql_stmt_bind_int(stmt, fromId);
    mysql_stmt_bind_int(stmt, amount);   // guard: never go negative
    mysql_transaction_add_stmt(tx, stmt);

    // Credit — reuse the same statement object with a different body
    mysql_stmt_close(stmt);
    stmt = mysql_stmt_new(g_MysqlConn,
        "UPDATE accounts SET balance = balance + ? WHERE id = ?");

    mysql_stmt_bind_int(stmt, amount);
    mysql_stmt_bind_int(stmt, toId);
    mysql_transaction_add_stmt(tx, stmt);
    mysql_stmt_close(stmt);

    // Audit row
    mysql_transaction_add(tx, "INSERT INTO transfer_log (at) VALUES (NOW())");

    // Executing CONSUMES the handle: tx is invalid from here on, and the batch
    // cannot be submitted twice by accident.
    mysql_transaction_execute(tx, "OnTransferDone", "dd", playerid, amount);
    return 1;
}

forward OnTransferDone(playerid, amount);
public OnTransferDone(playerid, amount)
{
    // The callback receives the cache of the LAST step in the batch.
    // If this fires at all, every step committed.
    printf("[mysql] transfer of %d committed for playerid %d", amount, playerid);
    SendClientMessage(playerid, 0x00FF00FF, "Transfer complete.");
    return 1;
}

// A failure anywhere in the batch rolls everything back and lands here instead.
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    if (strcmp(callback, "OnTransferDone") == 0)
    {
        printf("[mysql] transfer rolled back (error %d): %s", errorid, error);
    }
    return 1;
}

// --- Plain SQL steps ----------------------------------------------------------
//
// When no player input is involved, plain strings are fine.

ResetDailyStats()
{
    new tx = mysql_transaction_new(g_MysqlConn);
    if (tx == 0) return 0;

    mysql_transaction_add(tx, "UPDATE players SET daily_score = 0");
    mysql_transaction_add(tx, "DELETE FROM daily_rewards");
    mysql_transaction_add(tx, "INSERT INTO maintenance_log (task, at) VALUES ('daily_reset', NOW())");

    mysql_transaction_execute(tx, "OnDailyReset");
    return 1;
}

forward OnDailyReset();
public OnDailyReset()
{
    printf("[mysql] daily stats reset committed");
    return 1;
}

// --- Abandoning a batch -------------------------------------------------------
//
// If you build a transaction and then decide not to run it, destroy it —
// otherwise the handle stays around until the plugin unloads.
//
//     new tx = mysql_transaction_new(g_MysqlConn);
//     mysql_transaction_add(tx, "...");
//     if (somethingWentWrong)
//     {
//         mysql_transaction_destroy(tx);
//         return 0;
//     }
