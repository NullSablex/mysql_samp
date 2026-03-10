# Migracao do MySQL R41-4

Este guia ajuda na migracao de servidores que usam o plugin **MySQL R41-4** (BlueG/maddinat0r) para o **mysql_samp**.

Referencia completa do R41-4: `reference/a_mysql.inc`

## Diferencas principais

### Queries

| R41-4 | mysql_samp | Notas |
|---|---|---|
| `mysql_tquery(MySQL:handle, query, cb, fmt, ...)` | `mysql_query(connId, query, cb, fmt, ...)` | Threaded FIFO (mesmo comportamento) |
| `mysql_pquery(MySQL:handle, query, cb, fmt, ...)` | `mysql_pquery(connId, query, cb, fmt, ...)` | Paralela sem ordem (igual) |
| `Cache:mysql_query(MySQL:handle, query, use_cache)` | *(removido)* | Era sincrona — nao existe mais |
| `mysql_tquery_file(...)` | *(removido)* | Nao suportado |
| `mysql_query_file(...)` | *(removido)* | Nao suportado |

> No R41-4, `mysql_query` era sincrona e travava o servidor. No mysql_samp, `mysql_query` e sempre threaded (equivale ao antigo `mysql_tquery`).

### mysql_format: %s agora escapa

**Mudanca critica:** No R41-4, `%s` inseria a string raw (sem escape). No mysql_samp, `%s` **escapa automaticamente**.

| Especificador | R41-4 | mysql_samp |
|---|---|---|
| `%d` / `%i` | Inteiro | Inteiro (igual) |
| `%f` | Float | Float (igual) |
| `%s` | String **raw** (sem escape) | String **escaped** (com escape) |
| `%e` | String escaped | String escaped (alias de %s) |
| `%r` | *(nao existe)* | String raw (sem escape) |

**Migracao:**
- Se voce usava `%e` → mantenha `%e` ou troque por `%s` (ambos escapam)
- Se voce usava `%s` para inserir dados do usuario → nao precisa mudar (agora e seguro automaticamente)
- Se voce usava `%s` para inserir valores confiaveis (nomes de tabela, SQL dinamico) → troque por `%r`

```pawn
// R41-4
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM %s WHERE name = '%e'", tableName, playerName);

// mysql_samp
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM %r WHERE name = '%s'", tableName, playerName);
```

### mysql_escape_string

| R41-4 | mysql_samp |
|---|---|
| `mysql_escape_string(src, dest, max_len, MySQL:handle)` | `mysql_escape_string(src, dest, max_len)` |

No mysql_samp, o escape e uma funcao pura — nao requer conexao. O charset e sempre UTF-8 (forcado pelo plugin).

### Cache — mudanca de assinatura

**Mudanca mais importante:** no R41-4, natives de cache int/float/null escrevem por referencia (`&destination`). No mysql_samp, elas **retornam o valor diretamente**.

| R41-4 | mysql_samp | Mudanca |
|---|---|---|
| `cache_get_row_count(&dest)` | `cache_get_row_count()` retorna `int` | By-ref → retorno |
| `cache_get_field_count(&dest)` | `cache_get_field_count()` retorna `int` | By-ref → retorno |
| `cache_get_result_count(&dest)` | *(removido)* | Multi-result sets |
| `cache_get_value_index(row, col, dest[], max_len)` | `cache_get_value_index(row, col, dest[], max_len)` | Igual |
| `cache_get_value_index_int(row, col, &dest)` | `cache_get_value_index_int(row, col)` retorna `int` | By-ref → retorno |
| `cache_get_value_index_float(row, col, &Float:dest)` | `cache_get_value_index_float(row, col)` retorna `Float` | By-ref → retorno |
| `cache_get_value_name(row, name, dest[], max_len)` | `cache_get_value_name(row, name, dest[], max_len)` | Igual |
| `cache_get_value_name_int(row, name, &dest)` | `cache_get_value_name_int(row, name)` retorna `int` | By-ref → retorno |
| `cache_get_value_name_float(row, name, &Float:dest)` | `cache_get_value_name_float(row, name)` retorna `Float` | By-ref → retorno |
| `cache_is_value_index_null(row, col, &bool:dest)` | `cache_is_value_index_null(row, col)` retorna `bool` | By-ref → retorno |
| `cache_is_value_name_null(row, name, &bool:dest)` | `cache_is_value_name_null(row, name)` retorna `bool` | By-ref → retorno |
| `cache_set_result(idx)` | *(removido)* | Multi-result sets |
| `cache_save()` | `cache_save()` | Igual (sem tag `Cache:`) |
| `cache_delete(Cache:id)` | `cache_delete(id)` | Sem tag |
| `cache_set_active(Cache:id)` | `cache_set_active(id)` | Sem tag |
| `cache_unset_active()` | `cache_unset_active()` | Igual |
| `cache_affected_rows()` | `cache_affected_rows()` | Igual |
| `cache_insert_id()` | `cache_insert_id()` | Igual |
| `cache_warning_count()` | `cache_warning_count()` | Igual |
| `cache_get_query_exec_time(unit)` | `cache_get_query_exec_time()` | Sem param (sempre ms) |
| `cache_get_query_string(dest, max_len)` | `cache_get_query_string(dest, max_len)` | Igual |
| `cache_is_any_active()` | `cache_is_any_active()` | Igual |
| `cache_is_valid(Cache:id)` | `cache_is_valid(id)` | Sem tag |

**Stock wrappers do R41-4:**
- `cache_num_rows()` → usar `cache_get_row_count()` diretamente
- `cache_num_fields()` → usar `cache_get_field_count()` diretamente

### ORM

| R41-4 | mysql_samp | Diferenca |
|---|---|---|
| `ORM:orm_create(table, MySQL:handle)` | `orm_create(table, connId)` | Sem tags |
| `orm_destroy(ORM:id)` | `orm_destroy(orm_id)` retorna `bool` | Retorno adicionado |
| `E_ORM_ERROR:orm_errno(ORM:id)` | `orm_errno(orm_id)` retorna `int` | Sem tag |
| `orm_select(ORM:id, cb, fmt, ...)` | `orm_select(orm_id, cb, fmt, ...)` | Sem tag |
| `orm_update(ORM:id, cb, fmt, ...)` | `orm_update(orm_id, cb, fmt, ...)` | Sem tag |
| `orm_insert(ORM:id, cb, fmt, ...)` | `orm_insert(orm_id, cb, fmt, ...)` | Sem tag |
| `orm_delete(ORM:id, cb, fmt, ...)` | `orm_delete(orm_id, cb, fmt, ...)` | Sem tag |
| `orm_save(ORM:id, cb, fmt, ...)` | `orm_save(orm_id, cb, fmt, ...)` | Sem tag |
| `orm_load(...)` | *(removido)* | Era alias de orm_select |
| `orm_apply_cache(ORM:id, row, result_idx)` | `orm_apply_cache(orm_id, row)` | Sem result_idx |
| `orm_addvar_int(ORM:id, &var, col)` | `orm_addvar_int(orm_id, &var, col)` | Sem tag |
| `orm_addvar_float(ORM:id, &Float:var, col)` | `orm_addvar_float(orm_id, &Float:var, col)` | Sem tag |
| `orm_addvar_string(ORM:id, var[], max, col)` | `orm_addvar_string(orm_id, var[], max, col)` | max limitado a 4096 |
| `orm_delvar(ORM:id, col)` | `orm_delvar(orm_id, col)` | Sem tag |
| `orm_clear_vars(ORM:id)` | `orm_clear_vars(orm_id)` | Sem tag |
| `orm_setkey(ORM:id, col)` | `orm_setkey(orm_id, col)` | Sem tag |

**Enum de erro diferente:**

| R41-4 | mysql_samp |
|---|---|
| `ERROR_INVALID = 0` | *(nao existe)* |
| `ERROR_OK = 1` | `ORM_OK = 0` |
| `ERROR_NO_DATA = 2` | `ORM_NO_DATA = 1` |

### Conexao

| R41-4 | mysql_samp |
|---|---|
| `MySQL:mysql_connect(host, user, pass, db, MySQLOpt:opt)` | `mysql_connect(host, user, pass, db, options)` |
| `MySQLOpt:mysql_init_options()` | `mysql_options_new()` |
| `mysql_set_option(MySQLOpt:opt, E_MYSQL_OPTION:type, ...)` | `mysql_options_set_int(opt, type, val)` / `mysql_options_set_str(opt, type, val[])` |
| `mysql_close(MySQL:handle)` | `mysql_close(connId)` |
| `mysql_stat(dest, max_len, MySQL:handle)` | `mysql_status(connId, dest, max_len)` |
| `mysql_connect_file(file)` | *(removido)* |
| `mysql_global_options(type, val)` | *(removido)* |

**Diferenca de estilo:** no R41-4, o `MySQL:handle` e o ultimo parametro (opcional, default `MySQL:1`). No mysql_samp, o `connId` e o primeiro parametro.

### Erro

| R41-4 | mysql_samp | Diferenca |
|---|---|---|
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId)` | Sem tag, connId primeiro |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` | connId primeiro (era ultimo) |

### Charset

| R41-4 | mysql_samp | Diferenca |
|---|---|---|
| `mysql_set_charset(charset, MySQL:handle)` | `mysql_set_charset(connId, charset)` | connId primeiro |
| `mysql_get_charset(dest, max_len, MySQL:handle)` | `mysql_get_charset(connId, dest, max_len)` | connId primeiro |

### Log

| R41-4 | mysql_samp |
|---|---|
| `mysql_log(E_LOGLEVEL:level)` — bitflags (DEBUG=1, INFO=2, WARNING=4, ERROR=8) | `mysql_log(level)` — sequencial (NONE=0, ERROR=1, WARNING=2, INFO=3, ALL=4) |

### Formato de callback

O mysql_samp aceita tanto `"i"` quanto `"d"` para inteiros. Se voce ja usa `"i"`, nao precisa mudar.

| R41-4 | mysql_samp |
|---|---|
| `"i"` | `"d"` ou `"i"` (ambos funcionam) |
| `"f"` | `"f"` (igual) |
| `"s"` | `"s"` (igual) |

### Forward

| R41-4 | mysql_samp |
|---|---|
| `OnQueryError(errorid, error[], callback[], query[], MySQL:handle)` | `OnQueryError(errorid, error[], callback[], query[], connId)` |

## Funcoes removidas

| R41-4 | Razao da remocao |
|---|---|
| `Cache:mysql_query(handle, query, use_cache)` | Bloqueia o servidor — inaceitavel |
| `mysql_tquery_file(...)` / `mysql_query_file(...)` | Nao suportado |
| `mysql_connect_file(file)` | Nao suportado |
| `mysql_global_options(type, val)` | Nao aplicavel |
| `cache_get_result_count(&dest)` | Multi-result sets desnecessario para SA:MP |
| `cache_set_result(idx)` | Multi-result sets desnecessario para SA:MP |
| `orm_load(...)` | Era alias de `orm_select` |

## Passo a passo da migracao

### 1. Substitua o include

```pawn
// Antes
#include <a_mysql>

// Depois
#include <mysql_samp>
```

### 2. Renomeie as queries

```pawn
// Antes
mysql_tquery(gMysql, query, "OnData", "i", playerid);

// Depois
mysql_query(gMysql, query, "OnData", "d", playerid);
// ou (ambos funcionam)
mysql_query(gMysql, query, "OnData", "i", playerid);
```

### 3. Atualize mysql_format

```pawn
// Antes (R41-4: %s = raw, %e = escaped)
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM %s WHERE name = '%e'", table, name);

// Depois (mysql_samp: %r = raw, %s = escaped)
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM %r WHERE name = '%s'", table, name);
```

### 4. Atualize mysql_connect com options

```pawn
// Antes (R41-4: porta via mysql_set_option)
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:gMysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// Depois (mysql_samp: porta via mysql_options_set_int)
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new gMysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// Ou, se usa porta 3306 (padrao):
new gMysql = mysql_connect("127.0.0.1", "root", "pass", "db");
```

### 5. Atualize mysql_escape_string

```pawn
// Antes (R41-4: handle como ultimo param)
mysql_escape_string(input, escaped, sizeof(escaped), gMysql);

// Depois (mysql_samp: sem handle)
mysql_escape_string(input, escaped);
```

### 6. Adapte cache int/float de by-ref para retorno direto

```pawn
// Antes (R41-4: by-ref)
new score;
cache_get_value_name_int(0, "score", score);

// Depois (mysql_samp: retorno direto)
new score = cache_get_value_name_int(0, "score");
```

### 7. Adapte cache row_count / field_count

```pawn
// Antes (R41-4)
new rows = cache_num_rows();
// ou
new rows;
cache_get_row_count(rows);

// Depois (mysql_samp)
new rows = cache_get_row_count();
```

### 8. Remova todas as tags

```pawn
// Antes
new MySQL:gMysql;
new Cache:cache = cache_save();
new ORM:orm = orm_create("table", gMysql);

// Depois
new gMysql;
new cache = cache_save();
new orm = orm_create("table", gMysql);
```

### 9. Atualize mysql_error (ordem dos params)

```pawn
// Antes (R41-4: handle ultimo)
new errMsg[256];
mysql_error(errMsg, sizeof(errMsg), gMysql);

// Depois (mysql_samp: connId primeiro)
new errMsg[256];
mysql_error(gMysql, errMsg);
```

### 10. Compile e teste

Compile seu gamemode com o novo include e teste todas as funcionalidades. Verifique `logs/mysql.log` para erros.
