// 10_password_hashing.pwn — storing player passwords with Argon2id.
//
// Never store a password with MD5, SHA1, or MySQL's PASSWORD()/SHA2() helpers.
// Those are fast by design, which is the opposite of what password storage
// needs: a leaked table falls to commodity GPU cracking in hours.
//
// Argon2id is deliberately slow and memory-hard (~19 MiB and tens of
// milliseconds per hash by default). That is exactly why both natives here are
// non-blocking: the work runs on a small worker pool and the result comes back
// through a callback, so the server thread never stalls.
//
// Storage: the output is a PHC string, e.g.
//   $argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash...
// about 100 characters. Use VARCHAR(255).
//
// It ALREADY CONTAINS a random per-hash salt. Do not add a salt column, and do
// not reuse one — two players with the same password produce different hashes
// precisely so the table does not reveal that they match.
//
// Note the callback signature: the result is always the FIRST argument,
// followed by the extras described by the format string.

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

// Suggested schema:
//   CREATE TABLE accounts (
//       id       INT AUTO_INCREMENT PRIMARY KEY,
//       name     VARCHAR(24)  NOT NULL UNIQUE,
//       password VARCHAR(255) NOT NULL,       -- the whole PHC string
//       INDEX (name)
//   ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

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

// --- Registration -------------------------------------------------------------

RegisterAccount(playerid, const password[])
{
    // Returns false if the queue is saturated or the password exceeds 1 KiB.
    if (!mysql_hash_password(password, "OnPasswordHashed", "d", playerid))
    {
        SendClientMessage(playerid, 0xFF0000FF, "Server busy, please try again.");
    }
    return 1;
}

// The hash arrives as the FIRST parameter, then the extras ("d" -> playerid).
forward OnPasswordHashed(const hash[], playerid);
public OnPasswordHashed(const hash[], playerid)
{
    if (hash[0] == EOS)
    {
        // Hashing failed; the reason is in logs/mysql.log.
        SendClientMessage(playerid, 0xFF0000FF, "Registration failed, try again.");
        return 1;
    }

    new name[MAX_PLAYER_NAME + 1];
    GetPlayerName(playerid, name, sizeof(name));

    // Store it with a prepared statement so neither the name nor the hash is
    // ever pasted into SQL text.
    new stmt = mysql_stmt_new(g_MysqlConn,
        "INSERT INTO accounts (name, password) VALUES (?, ?)");

    if (stmt == 0) return 1;

    mysql_stmt_bind_str(stmt, name);
    mysql_stmt_bind_str(stmt, hash);
    mysql_stmt_execute(stmt, "OnAccountCreated", "d", playerid);
    mysql_stmt_close(stmt);
    return 1;
}

forward OnAccountCreated(playerid);
public OnAccountCreated(playerid)
{
    printf("[mysql] account created, id=%d", cache_insert_id());
    SendClientMessage(playerid, 0x00FF00FF, "Account created. You are logged in.");
    return 1;
}

// --- Login --------------------------------------------------------------------
//
// Two steps: fetch the stored hash, then verify against it. The plaintext never
// touches SQL, so it never reaches logs/mysql.log or the server's query log.

new g_LoginAttempt[MAX_PLAYERS][129];

AttemptLogin(playerid, const password[])
{
    // Hold the attempt until the stored hash comes back.
    strcat((g_LoginAttempt[playerid][0] = EOS, g_LoginAttempt[playerid]), password, 129);

    new name[MAX_PLAYER_NAME + 1];
    GetPlayerName(playerid, name, sizeof(name));

    new stmt = mysql_stmt_new(g_MysqlConn, "SELECT password FROM accounts WHERE name = ?");
    if (stmt == 0) return 0;

    mysql_stmt_bind_str(stmt, name);
    mysql_stmt_execute(stmt, "OnLoginHashFetched", "d", playerid);
    mysql_stmt_close(stmt);
    return 1;
}

forward OnLoginHashFetched(playerid);
public OnLoginHashFetched(playerid)
{
    if (cache_get_row_count() == 0)
    {
        // Unknown account. Real code should still burn roughly the same amount
        // of time here as a real verification would, otherwise response timing
        // tells an attacker which names exist.
        g_LoginAttempt[playerid][0] = EOS;
        SendClientMessage(playerid, 0xFF0000FF, "Invalid name or password.");
        return 1;
    }

    new storedHash[256];
    cache_get_value_name(0, "password", storedHash);

    mysql_verify_password(g_LoginAttempt[playerid], storedHash, "OnPasswordVerified", "d", playerid);
    g_LoginAttempt[playerid][0] = EOS;   // do not keep the plaintext around
    return 1;
}

// The bool result arrives FIRST, then the extras.
forward OnPasswordVerified(bool:success, playerid);
public OnPasswordVerified(bool:success, playerid)
{
    if (!success)
    {
        SendClientMessage(playerid, 0xFF0000FF, "Invalid name or password.");
        return 1;
    }

    SendClientMessage(playerid, 0x00FF00FF, "Logged in.");
    return 1;
}

public OnPlayerDisconnect(playerid, reason)
{
    g_LoginAttempt[playerid][0] = EOS;
    return 1;
}
