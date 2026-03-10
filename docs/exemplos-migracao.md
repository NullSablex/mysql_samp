# Exemplos de migracao: antes e depois

Exemplos reais de codigo Pawn mostrando o **antes** (R41-4) e o **depois** (mysql_samp).

---

## 1. Conexao basica

### R41-4
```pawn
new MySQL:gMysql;

public OnGameModeInit()
{
    gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp");

    if (gMysql == MYSQL_INVALID_HANDLE)
    {
        printf("Falha na conexao MySQL.");
        return 0;
    }
    return 1;
}
```

### mysql_samp
```pawn
new gMysql;

public OnGameModeInit()
{
    gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp");

    if (gMysql == 0)
    {
        printf("Falha na conexao MySQL.");
        return 0;
    }
    return 1;
}
```

> **Mudancas:** sem tag `MySQL:`, sem `MYSQL_INVALID_HANDLE` (usar `0`).

---

## 2. Conexao com porta customizada

### R41-4
```pawn
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp", opt);
```

### mysql_samp
```pawn
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp", opt);
```

> **Mudancas:** sem tags, `mysql_init_options` → `mysql_options_new`, `mysql_set_option` → `mysql_options_set_int`, `SERVER_PORT` → `MYSQL_OPT_PORT`.

---

## 3. Login — verificar se jogador existe

### R41-4
```pawn
public OnPlayerConnect(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(gMysql, query, sizeof(query),
        "SELECT * FROM jogadores WHERE nome = '%e'", name);
    mysql_tquery(gMysql, query, "OnCheckAccount", "i", playerid);
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

    mysql_format(gMysql, query, sizeof(query),
        "SELECT * FROM jogadores WHERE nome = '%s'", name);
    mysql_query(gMysql, query, "OnCheckAccount", "d", playerid);
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

> **Mudancas:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `"i"` → `"d"` (opcional), `cache_num_rows()` → `cache_get_row_count()`.

---

## 4. Login — carregar dados do jogador

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
        Player[playerid][PosX] = cache_get_value_name_float(0, "pos_x");
        Player[playerid][PosY] = cache_get_value_name_float(0, "pos_y");
        Player[playerid][PosZ] = cache_get_value_name_float(0, "pos_z");
        Player[playerid][Skin] = cache_get_value_name_int(0, "skin");
    }
    return 1;
}
```

> **Mudancas:** `cache_num_rows()` → `cache_get_row_count()`. `cache_get_value_name_int` mudou de by-ref (3 params) para retorno direto (2 params). `cache_get_value_name_float` idem. `cache_get_value_name` para strings manteve a mesma assinatura (usa `sizeof` automatico).

---

## 5. Registro — inserir jogador

### R41-4
```pawn
public OnDialogResponse(playerid, dialogid, response, listitem, inputtext[])
{
    if (dialogid == DIALOG_REGISTER && response)
    {
        new query[256], name[MAX_PLAYER_NAME], hash[65];
        GetPlayerName(playerid, name, sizeof(name));
        SHA256_PassHash(inputtext, SALT, hash, sizeof(hash));

        mysql_format(gMysql, query, sizeof(query),
            "INSERT INTO jogadores (nome, hash) VALUES ('%e', '%e')",
            name, hash);
        mysql_tquery(gMysql, query, "OnPlayerRegister", "i", playerid);
    }
    return 1;
}

forward OnPlayerRegister(playerid);
public OnPlayerRegister(playerid)
{
    Player[playerid][ID] = cache_insert_id();
    SendClientMessage(playerid, -1, "Registrado com sucesso!");
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

        mysql_format(gMysql, query, sizeof(query),
            "INSERT INTO jogadores (nome, hash) VALUES ('%s', '%s')",
            name, hash);
        mysql_query(gMysql, query, "OnPlayerRegister", "d", playerid);
    }
    return 1;
}

forward OnPlayerRegister(playerid);
public OnPlayerRegister(playerid)
{
    Player[playerid][ID] = cache_insert_id();
    SendClientMessage(playerid, -1, "Registrado com sucesso!");
    return 1;
}
```

> **Mudancas:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `"i"` → `"d"` (opcional).

---

## 6. Salvar dados do jogador

### R41-4
```pawn
stock SalvarJogador(playerid)
{
    new query[512];
    mysql_format(gMysql, query, sizeof(query),
        "UPDATE jogadores SET score = %d, money = %d, pos_x = %f, pos_y = %f, pos_z = %f WHERE id = %d",
        GetPlayerScore(playerid),
        GetPlayerMoney(playerid),
        Player[playerid][PosX],
        Player[playerid][PosY],
        Player[playerid][PosZ],
        Player[playerid][ID]);
    mysql_tquery(gMysql, query);
    return 1;
}
```

### mysql_samp
```pawn
stock SalvarJogador(playerid)
{
    new query[512];
    mysql_format(gMysql, query, sizeof(query),
        "UPDATE jogadores SET score = %d, money = %d, pos_x = %f, pos_y = %f, pos_z = %f WHERE id = %d",
        GetPlayerScore(playerid),
        GetPlayerMoney(playerid),
        Player[playerid][PosX],
        Player[playerid][PosY],
        Player[playerid][PosZ],
        Player[playerid][ID]);
    mysql_query(gMysql, query);
    return 1;
}
```

> **Mudanca minima:** apenas `mysql_tquery` → `mysql_query`. Sem callback, sem formato — fire-and-forget.

---

## 7. Sistema VIP — query com callback e parametros

### R41-4
```pawn
stock CheckarVIP(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(gMysql, query, sizeof(query),
        "SELECT vip_level, vip_expira FROM jogadores WHERE nome = '%e'", name);
    mysql_tquery(gMysql, query, "OnVIPCheck", "i", playerid);
}

forward OnVIPCheck(playerid);
public OnVIPCheck(playerid)
{
    if (cache_num_rows() > 0)
    {
        new level, expira[20];
        cache_get_value_name_int(0, "vip_level", level);
        cache_get_value_name(0, "vip_expira", expira, sizeof(expira));

        Player[playerid][VIPLevel] = level;
    }
    return 1;
}
```

### mysql_samp
```pawn
stock CheckarVIP(playerid)
{
    new query[128], name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    mysql_format(gMysql, query, sizeof(query),
        "SELECT vip_level, vip_expira FROM jogadores WHERE nome = '%s'", name);
    mysql_query(gMysql, query, "OnVIPCheck", "d", playerid);
}

forward OnVIPCheck(playerid);
public OnVIPCheck(playerid)
{
    if (cache_get_row_count() > 0)
    {
        new expira[20];
        Player[playerid][VIPLevel] = cache_get_value_name_int(0, "vip_level");
        cache_get_value_name(0, "vip_expira", expira);
    }
    return 1;
}
```

> **Mudancas:** `%e` → `%s`, `mysql_tquery` → `mysql_query`, `cache_num_rows` → `cache_get_row_count`, `cache_get_value_name_int` mudou de by-ref para retorno direto, `cache_get_value_name` sem `sizeof` explicito (usa padrao).

---

## 8. Sistema de ban

### R41-4
```pawn
stock BanirJogador(playerid, adminid, const razao[])
{
    new query[256], nome[MAX_PLAYER_NAME], admin[MAX_PLAYER_NAME];
    GetPlayerName(playerid, nome, sizeof(nome));
    GetPlayerName(adminid, admin, sizeof(admin));

    mysql_format(gMysql, query, sizeof(query),
        "INSERT INTO bans (nome, admin, razao, ip) VALUES ('%e', '%e', '%e', '%e')",
        nome, admin, razao, Player[playerid][IP]);
    mysql_tquery(gMysql, query);
    Kick(playerid);
}

stock VerificarBan(playerid)
{
    new query[128], nome[MAX_PLAYER_NAME];
    GetPlayerName(playerid, nome, sizeof(nome));

    mysql_format(gMysql, query, sizeof(query),
        "SELECT razao FROM bans WHERE nome = '%e'", nome);
    mysql_tquery(gMysql, query, "OnBanCheck", "i", playerid);
}

forward OnBanCheck(playerid);
public OnBanCheck(playerid)
{
    if (cache_num_rows() > 0)
    {
        new razao[128];
        cache_get_value_name(0, "razao", razao, sizeof(razao));
        SendClientMessage(playerid, -1, razao);
        Kick(playerid);
    }
    return 1;
}
```

### mysql_samp
```pawn
stock BanirJogador(playerid, adminid, const razao[])
{
    new query[256], nome[MAX_PLAYER_NAME], admin[MAX_PLAYER_NAME];
    GetPlayerName(playerid, nome, sizeof(nome));
    GetPlayerName(adminid, admin, sizeof(admin));

    mysql_format(gMysql, query, sizeof(query),
        "INSERT INTO bans (nome, admin, razao, ip) VALUES ('%s', '%s', '%s', '%s')",
        nome, admin, razao, Player[playerid][IP]);
    mysql_query(gMysql, query);
    Kick(playerid);
}

stock VerificarBan(playerid)
{
    new query[128], nome[MAX_PLAYER_NAME];
    GetPlayerName(playerid, nome, sizeof(nome));

    mysql_format(gMysql, query, sizeof(query),
        "SELECT razao FROM bans WHERE nome = '%s'", nome);
    mysql_query(gMysql, query, "OnBanCheck", "d", playerid);
}

forward OnBanCheck(playerid);
public OnBanCheck(playerid)
{
    if (cache_get_row_count() > 0)
    {
        new razao[128];
        cache_get_value_name(0, "razao", razao);
        SendClientMessage(playerid, -1, razao);
        Kick(playerid);
    }
    return 1;
}
```

---

## 9. Carregar veiculos do banco

### R41-4
```pawn
stock CarregarVeiculos()
{
    mysql_tquery(gMysql, "SELECT * FROM veiculos", "OnVeiculosCarregados");
}

forward OnVeiculosCarregados();
public OnVeiculosCarregados()
{
    new rows = cache_num_rows();

    for (new i = 0; i < rows; i++)
    {
        new modelo, Float:x, Float:y, Float:z, Float:a, cor1, cor2;

        cache_get_value_name_int(i, "modelo", modelo);
        cache_get_value_name_float(i, "pos_x", x);
        cache_get_value_name_float(i, "pos_y", y);
        cache_get_value_name_float(i, "pos_z", z);
        cache_get_value_name_float(i, "angulo", a);
        cache_get_value_name_int(i, "cor1", cor1);
        cache_get_value_name_int(i, "cor2", cor2);

        CreateVehicle(modelo, x, y, z, a, cor1, cor2, -1);
    }
    printf("Veiculos carregados: %d", rows);
    return 1;
}
```

### mysql_samp
```pawn
stock CarregarVeiculos()
{
    mysql_query(gMysql, "SELECT * FROM veiculos", "OnVeiculosCarregados");
}

forward OnVeiculosCarregados();
public OnVeiculosCarregados()
{
    new rows = cache_get_row_count();

    for (new i = 0; i < rows; i++)
    {
        new modelo = cache_get_value_name_int(i, "modelo");
        new Float:x = cache_get_value_name_float(i, "pos_x");
        new Float:y = cache_get_value_name_float(i, "pos_y");
        new Float:z = cache_get_value_name_float(i, "pos_z");
        new Float:a = cache_get_value_name_float(i, "angulo");
        new cor1 = cache_get_value_name_int(i, "cor1");
        new cor2 = cache_get_value_name_int(i, "cor2");

        CreateVehicle(modelo, x, y, z, a, cor1, cor2, -1);
    }
    printf("Veiculos carregados: %d", rows);
    return 1;
}
```

> **Mudancas:** `mysql_tquery` → `mysql_query`, `cache_num_rows` → `cache_get_row_count`, `cache_get_value_name_int/float` mudou de by-ref (3 params) para retorno direto (2 params).

---

## 10. ORM — sistema de jogador completo

### R41-4
```pawn
enum pInfo
{
    ORM:ORM_ID,
    ID,
    Nome[MAX_PLAYER_NAME],
    Hash[65],
    Score,
    Money,
    Float:PosX,
    Float:PosY,
    Float:PosZ,
};
new Player[MAX_PLAYERS][pInfo];

stock CriarORM(playerid)
{
    new ORM:orm = orm_create("jogadores", gMysql);
    Player[playerid][ORM_ID] = orm;

    orm_addvar_int(orm, Player[playerid][ID], "id");
    orm_addvar_string(orm, Player[playerid][Nome], MAX_PLAYER_NAME, "nome");
    orm_addvar_string(orm, Player[playerid][Hash], 65, "hash");
    orm_addvar_int(orm, Player[playerid][Score], "score");
    orm_addvar_int(orm, Player[playerid][Money], "money");
    orm_addvar_float(orm, Player[playerid][PosX], "pos_x");
    orm_addvar_float(orm, Player[playerid][PosY], "pos_y");
    orm_addvar_float(orm, Player[playerid][PosZ], "pos_z");

    orm_setkey(orm, "id");
    return _:orm;
}

// Carregar
orm_select(Player[playerid][ORM_ID], "OnPlayerLoad", "i", playerid);

forward OnPlayerLoad(playerid);
public OnPlayerLoad(playerid)
{
    orm_apply_cache(Player[playerid][ORM_ID], 0);
    SetPlayerScore(playerid, Player[playerid][Score]);
    GivePlayerMoney(playerid, Player[playerid][Money]);
}

// Salvar
orm_save(Player[playerid][ORM_ID]);
```

### mysql_samp
```pawn
enum pInfo
{
    ORM_ID,
    ID,
    Nome[MAX_PLAYER_NAME],
    Hash[65],
    Score,
    Money,
    Float:PosX,
    Float:PosY,
    Float:PosZ,
};
new Player[MAX_PLAYERS][pInfo];

stock CriarORM(playerid)
{
    new orm = orm_create("jogadores", gMysql);
    Player[playerid][ORM_ID] = orm;

    orm_addvar_int(orm, Player[playerid][ID], "id");
    orm_addvar_string(orm, Player[playerid][Nome], MAX_PLAYER_NAME, "nome");
    orm_addvar_string(orm, Player[playerid][Hash], 65, "hash");
    orm_addvar_int(orm, Player[playerid][Score], "score");
    orm_addvar_int(orm, Player[playerid][Money], "money");
    orm_addvar_float(orm, Player[playerid][PosX], "pos_x");
    orm_addvar_float(orm, Player[playerid][PosY], "pos_y");
    orm_addvar_float(orm, Player[playerid][PosZ], "pos_z");

    orm_setkey(orm, "id");
    return orm;
}

// Carregar
orm_select(Player[playerid][ORM_ID], "OnPlayerLoad", "d", playerid);

forward OnPlayerLoad(playerid);
public OnPlayerLoad(playerid)
{
    orm_apply_cache(Player[playerid][ORM_ID], 0);
    SetPlayerScore(playerid, Player[playerid][Score]);
    GivePlayerMoney(playerid, Player[playerid][Money]);
}

// Salvar
orm_save(Player[playerid][ORM_ID]);
```

> **Mudancas:** sem tag `ORM:` (usar `int`), `"i"` → `"d"` (opcional). O resto e identico.

---

## 11. Cache salvo — guardar e restaurar

### R41-4
```pawn
forward OnQueryA(playerid);
public OnQueryA(playerid)
{
    new Cache:cache = cache_save();

    mysql_tquery(gMysql, "SELECT ...", "OnQueryB", "ii", playerid, _:cache);
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

    mysql_query(gMysql, "SELECT ...", "OnQueryB", "dd", playerid, cache);
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

> **Mudancas:** sem `Cache:`, `mysql_tquery` → `mysql_query`, `"ii"` → `"dd"` (opcional — `"ii"` tambem funciona), `cache_num_rows` → `cache_get_row_count`.

---

## 12. Tratamento de erros

### R41-4
```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], MySQL:handle);
public OnQueryError(errorid, const error[], const callback[], const query[], MySQL:handle)
{
    printf("[MySQL] Erro %d na query: %s", errorid, error);
    printf("[MySQL] Callback: %s | Query: %s", callback, query);
    return 1;
}
```

### mysql_samp
```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[MySQL] Erro %d na query: %s", errorid, error);
    printf("[MySQL] Callback: %s | Query: %s", callback, query);

    // Novidade: consultar erro detalhado da conexao
    new errMsg[256];
    mysql_error(connId, errMsg);
    printf("[MySQL] Detalhe: %s", errMsg);
    return 1;
}
```

> **Mudanca:** sem tag `MySQL:` no ultimo parametro. Bonus: `mysql_error(connId, dest)` para detalhes extras.

---

## 13. Escape de strings

### R41-4
```pawn
new escaped[128];
mysql_escape_string(input, escaped, sizeof(escaped), gMysql);
```

### mysql_samp
```pawn
new escaped[128];
mysql_escape_string(input, escaped);
```

> **Mudancas:** sem handle MySQL como ultimo parametro. No mysql_samp o escape e sempre UTF-8, nao depende de conexao.

---

## 14. NULL check

### R41-4
```pawn
new bool:is_null;
cache_is_value_name_null(0, "email", is_null);
if (is_null)
{
    // campo e NULL
}
```

### mysql_samp
```pawn
if (cache_is_value_name_null(0, "email"))
{
    // campo e NULL
}
```

> **Mudanca:** retorno direto em vez de by-ref. Mais limpo.

---

## Checklist rapido de busca e substituicao

Para migrar mecanicamente, busque e substitua no seu gamemode:

| Buscar | Substituir por |
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
| `Cache:` (em chamadas) | *(remover)* |
| `ORM:` (em chamadas) | *(remover)* |
| `MySQL:` (em chamadas) | *(remover)* |
| `MySQLOpt:` (em chamadas) | *(remover)* |
| `MYSQL_INVALID_HANDLE` | `0` |
| `MYSQL_DEFAULT_HANDLE` | *(remover — usar variavel diretamente)* |

> **Requer revisao manual:**
> - `cache_get_value_*_int` e `_float`: mudar de 3 params (by-ref) para 2 params (retorno direto)
> - `cache_is_value_*_null`: mudar de 3 params (by-ref) para 2 params (retorno direto)
> - `cache_get_row_count` / `cache_get_field_count`: mudar de by-ref para retorno direto
> - `mysql_escape_string`: remover `MySQL:handle` (ultimo param)
> - `mysql_error`: inverter ordem — `mysql_error(connId, dest)` em vez de `mysql_error(dest, max_len, handle)`
> - `mysql_set_option` → `mysql_options_set_int` / `mysql_options_set_str`
> - `%s` → `%r` onde o `%s` antigo era intencional (raw). `%e` → `%s` ou manter `%e`
