-- Schema fixture for 11_config_and_scripts.pwn.
--
-- Run with:  mysql_query_file(g_mysql, "schema.sql", "OnSchemaReady");
--
-- Statements are split on `;` outside string literals and comments, so the
-- semicolons inside the comments and the string below do NOT split anything.
-- This file deliberately exercises that: if the splitter were naive, it would
-- produce broken fragments and the script would fail.

CREATE TABLE IF NOT EXISTS accounts (
    id       INT AUTO_INCREMENT PRIMARY KEY,
    name     VARCHAR(24)  NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,   -- Argon2id PHC string; see example 10
    score    INT          NOT NULL DEFAULT 0,
    INDEX (score)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS audit_log (
    id      INT AUTO_INCREMENT PRIMARY KEY,
    note    VARCHAR(255) NOT NULL,
    at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

# A hash-style comment is also skipped by the splitter.

/* A block comment containing a semicolon ; and a quote ' — neither ends
   a statement, and the scanner walks past both. */

INSERT INTO audit_log (note) VALUES ('schema applied; version 1');
