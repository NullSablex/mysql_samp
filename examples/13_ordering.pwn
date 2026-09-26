// 13_ordering.pwn - what is ordered, what is not, and what to use instead.
//
// mysql_query is described as FIFO. What that orders is the DELIVERY OF
// CALLBACKS, in submission order. It does NOT order execution: each query
// gets its own worker thread and its own pool connection as soon as it is
// submitted, so submitted statements run at the same time on the server.
//
// This matters the moment one statement depends on another.

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

    // WRONG - these race. The INSERT usually fails with "table doesn't exist",
    // because the CREATE is still running on another connection.
    //
    //     mysql_query(g_MysqlConn, "CREATE TABLE stats (id INT, kills INT)");
    //     mysql_query(g_MysqlConn, "INSERT INTO stats VALUES (1, 0)");

    // RIGHT for schema work: one file, statements run in order, one
    // connection, still non-blocking, still no callback needed.
    // The path is relative to the server root, not to scriptfiles/.
    mysql_query_file(g_MysqlConn, "schema.sql");
    return 1;
}

// RIGHT for writes that depend on each other: a transaction runs its steps in
// order on one connection, and rolls back if any of them fails.
StorePurchase(playerid, itemid, price)
{
    new tx = mysql_transaction_new(g_MysqlConn);
    if (tx == 0)
        return 0;

    new query[192];

    mysql_format(g_MysqlConn, query, sizeof(query),
                 "UPDATE players SET money = money - %d WHERE id = %d", price, playerid);
    mysql_transaction_add(tx, query);

    mysql_format(g_MysqlConn, query, sizeof(query),
                 "INSERT INTO inventory (player_id, item_id) VALUES (%d, %d)", playerid, itemid);
    mysql_transaction_add(tx, query);

    // No callback here either: either both steps land, or neither does.
    mysql_transaction_execute(tx);
    return 1;
}

// RIGHT for a read that depends on a write: chain it in the write's callback.
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[MySQL] error %d on '%s': %s", errorid, query, error);
    return 1;
}
