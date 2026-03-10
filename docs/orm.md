# ORM

O ORM (Object-Relational Mapping) mapeia variaveis Pawn para colunas de uma tabela no banco de dados. Com ele voce pode fazer SELECT, INSERT, UPDATE e DELETE sem escrever SQL manualmente.

## Conceito

1. Crie uma instancia ORM vinculada a uma tabela e conexao
2. Associe variaveis Pawn as colunas (bindings)
3. Defina a coluna chave primaria
4. Execute operacoes CRUD — o SQL e gerado automaticamente

## Criacao e destruicao

### orm_create

```pawn
native orm_create(const table[], connId);
```

**Retorno:** ID do ORM (>= 1) ou `0` se a conexao for invalida.

### orm_destroy

```pawn
native bool:orm_destroy(orm_id);
```

```pawn
new ORM:ormPlayer;

public OnGameModeInit() {
    ormPlayer = ORM:orm_create("players", gMysql);
    // ... configurar bindings ...
}

public OnGameModeExit() {
    orm_destroy(_:ormPlayer);
}
```

## Variable bindings

Associe variaveis Pawn as colunas do banco:

### orm_addvar_int

```pawn
native bool:orm_addvar_int(orm_id, &var, const column_name[]);
```

### orm_addvar_float

```pawn
native bool:orm_addvar_float(orm_id, &Float:var, const column_name[]);
```

### orm_addvar_string

```pawn
native bool:orm_addvar_string(orm_id, var[], var_max_len, const column_name[]);
```

> `var_max_len` deve estar entre 1 e 4096. Valores fora deste intervalo sao rejeitados.

### orm_delvar / orm_clear_vars

```pawn
native bool:orm_delvar(orm_id, const column_name[]);
native bool:orm_clear_vars(orm_id);
```

### orm_setkey

Define a coluna de chave primaria (necessaria para SELECT, UPDATE, DELETE).

```pawn
native bool:orm_setkey(orm_id, const column_name[]);
```

### Exemplo completo de setup

```pawn
enum PlayerData {
    pId,
    pName[MAX_PLAYER_NAME],
    Float:pScore,
    pLevel
}

new gPlayerData[MAX_PLAYERS][PlayerData];
new ORM:gPlayerORM[MAX_PLAYERS];

stock SetupPlayerORM(playerid) {
    new oid = orm_create("players", gMysql);
    gPlayerORM[playerid] = ORM:oid;

    orm_addvar_int(oid, gPlayerData[playerid][pId], "id");
    orm_addvar_string(oid, gPlayerData[playerid][pName], MAX_PLAYER_NAME, "name");
    orm_addvar_float(oid, gPlayerData[playerid][pScore], "score");
    orm_addvar_int(oid, gPlayerData[playerid][pLevel], "level");

    orm_setkey(oid, "id");
}
```

## Operacoes CRUD

Todas as operacoes sao **non-blocking** (usam `mysql_query` internamente).

### orm_select

Executa um SELECT usando o valor atual da chave primaria.

```pawn
native bool:orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
```

SQL gerado: `SELECT col1, col2, ... FROM tabela WHERE chave = valor`

```pawn
// Define o ID a buscar
gPlayerData[playerid][pId] = playerDBId;

// Executa SELECT
orm_select(_:gPlayerORM[playerid], "OnPlayerDataLoaded", "d", playerid);

forward OnPlayerDataLoaded(playerid);
public OnPlayerDataLoaded(playerid) {
    // Aplica os dados do cache nas variaveis vinculadas
    orm_apply_cache(_:gPlayerORM[playerid]);

    // Agora gPlayerData[playerid] tem os valores do banco
    printf("Nome: %s, Level: %d", gPlayerData[playerid][pName], gPlayerData[playerid][pLevel]);
}
```

### orm_insert

Insere um novo registro com os valores atuais das variaveis vinculadas.

```pawn
native bool:orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
```

SQL gerado: `INSERT INTO tabela (col1, col2, ...) VALUES (val1, val2, ...)`

```pawn
// Define os dados
gPlayerData[playerid][pName] = "NovoJogador";
gPlayerData[playerid][pLevel] = 1;
gPlayerData[playerid][pScore] = 0.0;

orm_insert(_:gPlayerORM[playerid], "OnPlayerInserted", "d", playerid);

forward OnPlayerInserted(playerid);
public OnPlayerInserted(playerid) {
    // Obtem o ID auto_increment gerado
    gPlayerData[playerid][pId] = cache_insert_id();
    printf("Jogador inserido com ID: %d", gPlayerData[playerid][pId]);
}
```

### orm_update

Atualiza o registro usando o valor atual da chave primaria.

```pawn
native bool:orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
```

SQL gerado: `UPDATE tabela SET col1=val1, col2=val2, ... WHERE chave = valor`

```pawn
// Modifica os dados
gPlayerData[playerid][pLevel] = 10;
gPlayerData[playerid][pScore] = 1500.0;

// Salva no banco
orm_update(_:gPlayerORM[playerid]);
```

### orm_delete

Remove o registro usando o valor atual da chave primaria.

```pawn
native bool:orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
```

SQL gerado: `DELETE FROM tabela WHERE chave = valor`

```pawn
orm_delete(_:gPlayerORM[playerid], "OnPlayerDeleted", "d", playerid);

forward OnPlayerDeleted(playerid);
public OnPlayerDeleted(playerid) {
    printf("Jogador %d removido do banco", playerid);
}
```

### orm_save

Decide automaticamente entre INSERT e UPDATE baseado no valor da chave primaria:
- Se a chave for `0` (int), `0.0` (float) ou vazia (string) → **INSERT**
- Caso contrario → **UPDATE**

```pawn
native bool:orm_save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
```

```pawn
// Se pId == 0, faz INSERT. Se pId > 0, faz UPDATE.
orm_save(_:gPlayerORM[playerid], "OnPlayerSaved", "d", playerid);
```

## orm_apply_cache

Escreve os valores do cache ativo nas variaveis Pawn vinculadas.

```pawn
native bool:orm_apply_cache(orm_id, row = 0);
```

| Parametro | Tipo | Descricao |
|---|---|---|
| `orm_id` | int | ID do ORM |
| `row` | int | Indice da linha do cache (padrao: 0) |

Deve ser chamado **dentro de um callback de query** (quando o cache esta ativo).

## orm_errno

Retorna o codigo de erro da ultima operacao ORM.

```pawn
native orm_errno(orm_id);
```

| Codigo | Constante | Descricao |
|---|---|---|
| 0 | `ORM_OK` | Sem erro |
| 1 | `ORM_NO_DATA` | SELECT nao retornou dados |

```pawn
forward OnPlayerLoaded(playerid);
public OnPlayerLoaded(playerid) {
    orm_apply_cache(_:gPlayerORM[playerid]);

    if (orm_errno(_:gPlayerORM[playerid]) == ORM_NO_DATA) {
        // Jogador nao encontrado — criar novo
        orm_insert(_:gPlayerORM[playerid]);
    }
}
```

## Seguranca

- Strings vinculadas sao **escapadas automaticamente** no SQL gerado
- Nomes de tabela e coluna sao sanitizados via `escape_identifier` (backticks removidos)
- ORMs sao destruidos automaticamente quando o AMX e descarregado (previne acesso a memoria invalida)
- `max_len` em `orm_addvar_string` e limitado a 4096 para prevenir escrita fora dos limites

## Exemplo completo: sistema de jogadores

```pawn
#include <a_samp>
#include <mysql_samp>

#define MAX_PLAYER_NAME 24

enum pInfo {
    pDBId,
    pName[MAX_PLAYER_NAME],
    pLevel,
    Float:pMoney
}

new PlayerInfo[MAX_PLAYERS][pInfo];
new ORM:PlayerORM[MAX_PLAYERS];
new gMysql;

public OnGameModeInit() {
    gMysql = mysql_connect("127.0.0.1", "root", "", "samp_server");

    if (mysql_errno()) {
        printf("MySQL: falha na conexao");
    }
    return 1;
}

public OnPlayerConnect(playerid) {
    // Cria ORM para o jogador
    new oid = orm_create("players", gMysql);
    PlayerORM[playerid] = ORM:oid;

    orm_addvar_int(oid, PlayerInfo[playerid][pDBId], "id");
    orm_addvar_string(oid, PlayerInfo[playerid][pName], MAX_PLAYER_NAME, "name");
    orm_addvar_int(oid, PlayerInfo[playerid][pLevel], "level");
    orm_addvar_float(oid, PlayerInfo[playerid][pMoney], "money");
    orm_setkey(oid, "id");

    // Busca pelo nome
    GetPlayerName(playerid, PlayerInfo[playerid][pName], MAX_PLAYER_NAME);

    new query[128];
    mysql_format(gMysql, query, sizeof(query),
        "SELECT * FROM players WHERE name = '%s' LIMIT 1",
        PlayerInfo[playerid][pName]
    );
    mysql_query(gMysql, query, "OnPlayerDataLoaded", "d", playerid);
    return 1;
}

forward OnPlayerDataLoaded(playerid);
public OnPlayerDataLoaded(playerid) {
    if (cache_get_row_count() > 0) {
        orm_apply_cache(_:PlayerORM[playerid]);
        printf("Jogador %s carregado (ID: %d, Level: %d)",
            PlayerInfo[playerid][pName],
            PlayerInfo[playerid][pDBId],
            PlayerInfo[playerid][pLevel]
        );
    } else {
        // Novo jogador — inserir
        PlayerInfo[playerid][pLevel] = 1;
        PlayerInfo[playerid][pMoney] = 0.0;
        orm_insert(_:PlayerORM[playerid], "OnPlayerCreated", "d", playerid);
    }
}

forward OnPlayerCreated(playerid);
public OnPlayerCreated(playerid) {
    PlayerInfo[playerid][pDBId] = cache_insert_id();
    printf("Novo jogador criado com ID: %d", PlayerInfo[playerid][pDBId]);
}

public OnPlayerDisconnect(playerid, reason) {
    // Salva dados e destroi ORM
    orm_save(_:PlayerORM[playerid]);
    orm_destroy(_:PlayerORM[playerid]);
    return 1;
}
```
