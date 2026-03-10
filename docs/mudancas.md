# O que mudou na pratica

Referencia rapida de tudo que muda ao migrar do **MySQL R41-4** para o **mysql_samp**.

## Include

```pawn
// R41-4
#include <a_mysql>

// mysql_samp
#include <mysql_samp>
```

## Tags removidas

O mysql_samp nao usa tags customizadas. Tudo e `int` simples.

| R41-4 | mysql_samp |
|---|---|
| `new MySQL:gMysql` | `new gMysql` |
| `new Cache:cache = cache_save();` | `new cache = cache_save();` |
| `new ORM:orm = orm_create(...)` | `new orm = orm_create(...)` |
| `cache_delete(Cache:cache);` | `cache_delete(cache);` |
| `MySQLOpt:opt = mysql_init_options()` | `new opt = mysql_options_new()` |

## Formato de callback

| R41-4 | mysql_samp | Significado |
|---|---|---|
| `"i"` | `"d"` ou `"i"` | Inteiro (ambos funcionam no mysql_samp) |
| `"f"` | `"f"` | Float (igual) |
| `"s"` | `"s"` | String (igual) |

> **Nota:** o mysql_samp aceita tanto `"d"` quanto `"i"` para inteiros. Se voce ja usa `"i"`, nao precisa mudar.

```pawn
// R41-4
mysql_tquery(gMysql, query, "OnLoad", "i", playerid);

// mysql_samp (ambos funcionam)
mysql_query(gMysql, query, "OnLoad", "d", playerid);
mysql_query(gMysql, query, "OnLoad", "i", playerid);
```

## mysql_format: especificadores

| Especificador | R41-4 | mysql_samp |
|---|---|---|
| `%d` / `%i` | Inteiro | Inteiro (igual) |
| `%f` | Float | Float (igual) |
| `%s` | String **raw** (sem escape) | String **escaped** (com escape) |
| `%e` | String escaped | String escaped (alias de %s) |
| `%r` | *(nao existe)* | String raw (sem escape) |

**Regra pratica:** onde voce usava `%e`, use `%s`. Onde usava `%s` para nomes de tabela/coluna, use `%r`.

## Queries

| R41-4 | mysql_samp | Comportamento |
|---|---|---|
| `Cache:mysql_query(handle, query, use_cache)` | *(removido)* | Era sincrona — nao existe mais |
| `mysql_tquery(handle, query, cb, fmt, ...)` | `mysql_query(connId, query, cb, fmt, ...)` | Threaded FIFO (mesmo comportamento) |
| `mysql_pquery(handle, query, cb, fmt, ...)` | `mysql_pquery(connId, query, cb, fmt, ...)` | Paralela sem ordem (igual) |

**Importante:** toda query e non-blocking. O cache so esta disponivel dentro do callback.

## Conexao

| R41-4 | mysql_samp |
|---|---|
| `MySQL:mysql_connect(host, user, pass, db, MySQLOpt:opt)` | `mysql_connect(host, user, pass, db, options)` |
| `MySQLOpt:mysql_init_options()` | `mysql_options_new()` |
| `mysql_set_option(MySQLOpt:opt, type, ...)` | `mysql_options_set_int(opt, type, val)` / `mysql_options_set_str(opt, type, val[])` |
| `mysql_escape_string(src, dest, max_len, MySQL:handle)` | `mysql_escape_string(src, dest, max_len)` |
| `mysql_stat(dest, max_len, MySQL:handle)` | `mysql_status(connId, dest, max_len)` |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` |
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId)` |
| `mysql_set_charset(charset, MySQL:handle)` | `mysql_set_charset(connId, charset)` |
| `mysql_get_charset(dest, max_len, MySQL:handle)` | `mysql_get_charset(connId, dest, max_len)` |
| `mysql_close(MySQL:handle)` | `mysql_close(connId)` |
| `mysql_unprocessed_queries(MySQL:handle)` | `mysql_unprocessed_queries()` |
| *(nao existe)* | `mysql_log(level)` |

**Diferenca de estilo:** no R41-4, o handle e o ultimo parametro (opcional, default `MySQL:1`). No mysql_samp, o connId e o primeiro parametro.

**Porta:** no R41-4, usa-se `mysql_set_option(opt, SERVER_PORT, 3307)`. No mysql_samp, usa-se `mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307)`.

```pawn
// R41-4
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:gMysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// mysql_samp
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new gMysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// mysql_samp (porta padrao 3306 — sem options)
new gMysql = mysql_connect("127.0.0.1", "root", "pass", "db");
```

## Cache — mudanca de assinatura (by-ref → retorno direto)

**Mudanca mais significativa:** no R41-4, a maioria das natives de cache escrevem o valor por referencia (`&destination`). No mysql_samp, elas **retornam o valor diretamente**.

### Contagem de linhas/colunas

| R41-4 | mysql_samp |
|---|---|
| `cache_get_row_count(&dest)` | `cache_get_row_count()` retorna `int` |
| `cache_get_field_count(&dest)` | `cache_get_field_count()` retorna `int` |
| `cache_num_rows()` (stock wrapper) | `cache_get_row_count()` |
| `cache_num_fields()` (stock wrapper) | `cache_get_field_count()` |

```pawn
// R41-4
new rows;
cache_get_row_count(rows);
// ou
new rows = cache_num_rows();

// mysql_samp
new rows = cache_get_row_count();
```

### Leitura de valores por indice

| R41-4 | mysql_samp |
|---|---|
| `cache_get_value_index(row, col, dest[], max_len)` | `cache_get_value_index(row, col, dest[], max_len)` (igual) |
| `cache_get_value_index_int(row, col, &dest)` | `cache_get_value_index_int(row, col)` retorna `int` |
| `cache_get_value_index_float(row, col, &Float:dest)` | `cache_get_value_index_float(row, col)` retorna `Float` |

```pawn
// R41-4
new score;
cache_get_value_index_int(0, 2, score);

// mysql_samp
new score = cache_get_value_index_int(0, 2);
```

### Leitura de valores por nome

| R41-4 | mysql_samp |
|---|---|
| `cache_get_value_name(row, col_name, dest[], max_len)` | `cache_get_value_name(row, col_name, dest[], max_len)` (igual) |
| `cache_get_value_name_int(row, col_name, &dest)` | `cache_get_value_name_int(row, col_name)` retorna `int` |
| `cache_get_value_name_float(row, col_name, &Float:dest)` | `cache_get_value_name_float(row, col_name)` retorna `Float` |

```pawn
// R41-4
new score;
cache_get_value_name_int(0, "score", score);
new Float:pos_x;
cache_get_value_name_float(0, "pos_x", pos_x);

// mysql_samp
new score = cache_get_value_name_int(0, "score");
new Float:pos_x = cache_get_value_name_float(0, "pos_x");
```

### NULL check

| R41-4 | mysql_samp |
|---|---|
| `cache_is_value_index_null(row, col, &bool:dest)` | `cache_is_value_index_null(row, col)` retorna `bool` |
| `cache_is_value_name_null(row, col_name, &bool:dest)` | `cache_is_value_name_null(row, col_name)` retorna `bool` |

```pawn
// R41-4
new bool:is_null;
cache_is_value_name_null(0, "email", is_null);

// mysql_samp
new bool:is_null = cache_is_value_name_null(0, "email");
```

## Cache — natives que mantiveram assinatura

- `cache_get_value_index(row, col, dest[], max_len)` — string por indice
- `cache_get_value_name(row, col_name, dest[], max_len)` — string por nome
- `cache_get_field_name(idx, dest[], max_len)` — nome da coluna
- `cache_affected_rows()` — rows afetadas
- `cache_insert_id()` — ultimo ID inserido
- `cache_warning_count()` — numero de warnings
- `cache_save()` — salva cache (retorna ID)
- `cache_delete(cache_id)` — remove cache salvo
- `cache_set_active(cache_id)` — ativa cache salvo
- `cache_unset_active()` — desativa cache manual
- `cache_is_any_active()` — verifica se ha cache ativo
- `cache_is_valid(cache_id)` — verifica se cache existe
- `cache_get_query_string(dest[], max_len)` — query original

## Cache — natives novas

| Native | Descricao |
|---|---|
| `cache_get_field_type(idx)` | Tipo MySQL da coluna |
| `cache_get_query_exec_time()` | Tempo de execucao em ms |

## Cache — natives removidas

| R41-4 | Razao |
|---|---|
| `cache_get_result_count(&dest)` | Multi-result sets desnecessario para SA:MP |
| `cache_set_result(idx)` | Multi-result sets desnecessario para SA:MP |
| `cache_get_query_exec_time(unit)` | Substituido por versao simplificada (sempre ms) |

## ORM

A API e similar ao R41-4, com diferencas:

| R41-4 | mysql_samp | Diferenca |
|---|---|---|
| `ORM:orm_create(table, MySQL:handle)` | `orm_create(table, connId)` | Sem tags |
| `orm_destroy(ORM:id)` | `orm_destroy(orm_id)` retorna `bool` | Retorno adicionado |
| `E_ORM_ERROR:orm_errno(ORM:id)` | `orm_errno(orm_id)` retorna `int` | Sem tag de enum |
| `orm_apply_cache(ORM:id, row, result_idx)` | `orm_apply_cache(orm_id, row)` | Sem result_idx |
| `orm_load(...)` | *(removido)* | Alias de orm_select |

**Enum de erro diferente:**

| R41-4 | mysql_samp |
|---|---|
| `ERROR_INVALID = 0` | *(nao existe)* |
| `ERROR_OK = 1` | `ORM_OK = 0` |
| `ERROR_NO_DATA = 2` | `ORM_NO_DATA = 1` |

Demais natives ORM (select, update, insert, delete, save, addvar_*, delvar, clear_vars, setkey) tem a mesma assinatura, apenas sem tags `ORM:`.

## Erro

| R41-4 | mysql_samp | Diferenca |
|---|---|---|
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId)` | Sem tag, connId primeiro |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` | connId primeiro (era ultimo) |

## Log

| R41-4 | mysql_samp |
|---|---|
| `mysql_log(E_LOGLEVEL:level)` | `mysql_log(level)` |
| Bitflags: DEBUG=1, INFO=2, WARNING=4, ERROR=8 | Sequencial: NONE=0, ERROR=1, WARNING=2, INFO=3, ALL=4 |

```pawn
// R41-4
mysql_log(ERROR | WARNING);

// mysql_samp
mysql_log(MYSQL_LOG_WARNING); // inclui ERROR e WARNING
```

## Forward

| R41-4 | mysql_samp |
|---|---|
| `OnQueryError(errorid, error[], callback[], query[], MySQL:handle)` | `OnQueryError(errorid, error[], callback[], query[], connId)` |

Mesma assinatura, sem tag `MySQL:`.

## Resumo das mudancas obrigatorias

1. Trocar `#include <a_mysql>` por `#include <mysql_samp>`
2. Trocar `mysql_tquery` por `mysql_query`
3. Trocar `%s` (raw) por `%r` no `mysql_format` onde usava strings nao-escapadas
4. Trocar `mysql_escape_string(..., MySQL:handle)` por `mysql_escape_string(src, dest, max_len)` (sem handle)
5. Trocar `cache_num_rows()` por `cache_get_row_count()`
6. Adaptar `cache_get_value_*_int` e `cache_get_value_*_float` de by-ref para retorno direto
7. Adaptar `cache_is_value_*_null` de by-ref para retorno direto
8. Adaptar `cache_get_row_count` e `cache_get_field_count` de by-ref para retorno direto
9. Remover todas as tags (`MySQL:`, `Cache:`, `ORM:`, `MySQLOpt:`, `E_ORM_ERROR:`)
10. Trocar `mysql_init_options()` por `mysql_options_new()`
11. Trocar `mysql_set_option(opt, type, ...)` por `mysql_options_set_int/set_str`
12. Inverter ordem dos params em `mysql_error` (connId agora e primeiro)
13. Trocar `mysql_stat` por `mysql_status` (connId primeiro)
14. Mover codigo que acessava cache apos query sincrona para dentro de callbacks

**Mudancas opcionais:**
- `"i"` → `"d"` nos formatos de callback (ambos funcionam no mysql_samp)
