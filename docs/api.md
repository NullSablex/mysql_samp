# Referencia da API

Tabela completa de todas as natives e forwards do mysql_samp.

## Conexao

| Native | Retorno | Descricao |
|---|---|---|
| `mysql_connect(host[], user[], pass[], db[], options = 0)` | int | Conecta ao MySQL. Retorna connId ou 0 |
| `mysql_close(connId)` | bool | Fecha conexao |
| `mysql_status(connId, dest[], max_len)` | bool | Metricas do servidor |
| `mysql_set_charset(connId, charset[])` | bool | Define charset (`SET NAMES`) |
| `mysql_get_charset(connId, dest[], max_len)` | bool | Obtem charset atual |

## Options

| Native | Retorno | Descricao |
|---|---|---|
| `mysql_options_new()` | int | Cria handle de options |
| `mysql_options_set_int(handle, option, value)` | bool | Define opcao numerica |
| `mysql_options_set_str(handle, option, value[])` | bool | Define opcao string |

### Opcoes disponiveis

| Constante | Tipo | Padrao | Descricao |
|---|---|---|---|
| `MYSQL_OPT_PORT` | int | 3306 | Porta TCP |
| `MYSQL_OPT_SSL` | int | false | Ativar TLS |
| `MYSQL_OPT_SSL_CA` | string | — | Certificado CA |
| `MYSQL_OPT_CONNECT_TIMEOUT` | int | — | Timeout (segundos) |

## Erro

| Native | Retorno | Descricao |
|---|---|---|
| `mysql_errno(connId = 0)` | int | Codigo do ultimo erro |
| `mysql_error(connId, dest[], max_len)` | bool | Mensagem do ultimo erro |

## Queries

| Native | Retorno | Descricao |
|---|---|---|
| `mysql_query(connId, query[], callback[], format[], ...)` | bool | Query FIFO com callback |
| `mysql_pquery(connId, query[], callback[], format[], ...)` | bool | Query paralela |
| `mysql_escape_string(src[], dest[], max_len)` | bool | Escape de string SQL |
| `mysql_format(connId, dest[], max_len, format[], ...)` | int | Formatacao printf-like |

### Especificadores de mysql_format

| Spec | Tipo | Descricao |
|---|---|---|
| `%d` / `%i` | int | Inteiro |
| `%f` | float | Decimal (4 casas) |
| `%s` / `%e` | string | String com escape automatico |
| `%r` | string | String raw (sem escape) |
| `%%` | — | Literal `%` |

## Cache — Leitura

| Native | Retorno | Descricao |
|---|---|---|
| `cache_get_row_count()` | int | Numero de linhas (-1 se sem cache) |
| `cache_get_field_count()` | int | Numero de colunas (-1 se sem cache) |
| `cache_get_field_name(idx, dest[], max_len)` | bool | Nome da coluna |
| `cache_get_field_type(idx)` | int | Tipo MySQL da coluna |
| `cache_get_value_index(row, col, dest[], max_len)` | bool | Valor string por indice |
| `cache_get_value_index_int(row, col)` | int | Valor int por indice |
| `cache_get_value_index_float(row, col)` | float | Valor float por indice |
| `cache_get_value_name(row, name[], dest[], max_len)` | bool | Valor string por nome |
| `cache_get_value_name_int(row, name[])` | int | Valor int por nome |
| `cache_get_value_name_float(row, name[])` | float | Valor float por nome |
| `cache_is_value_index_null(row, col)` | bool | NULL por indice |
| `cache_is_value_name_null(row, name[])` | bool | NULL por nome |

## Cache — Metadados

| Native | Retorno | Descricao |
|---|---|---|
| `cache_affected_rows()` | int | Linhas afetadas |
| `cache_insert_id()` | int | Last insert ID |
| `cache_warning_count()` | int | Warnings do MySQL |
| `cache_get_query_exec_time()` | int | Tempo de execucao (ms) |
| `cache_get_query_string(dest[], max_len)` | bool | Query original |

## Cache — Persistencia

| Native | Retorno | Descricao |
|---|---|---|
| `cache_save()` | int | Salva cache, retorna ID |
| `cache_delete(cache_id)` | bool | Remove cache salvo |
| `cache_set_active(cache_id)` | bool | Ativa cache salvo |
| `cache_unset_active()` | bool | Desativa cache manual |
| `cache_is_any_active()` | bool | Algum cache esta ativo? |
| `cache_is_valid(cache_id)` | bool | Cache salvo existe? |

## ORM

| Native | Retorno | Descricao |
|---|---|---|
| `orm_create(table[], connId)` | int | Cria instancia ORM |
| `orm_destroy(orm_id)` | bool | Destroi instancia |
| `orm_errno(orm_id)` | int | Ultimo erro ORM |
| `orm_select(orm_id, callback[], format[], ...)` | bool | SELECT non-blocking |
| `orm_update(orm_id, callback[], format[], ...)` | bool | UPDATE non-blocking |
| `orm_insert(orm_id, callback[], format[], ...)` | bool | INSERT non-blocking |
| `orm_delete(orm_id, callback[], format[], ...)` | bool | DELETE non-blocking |
| `orm_save(orm_id, callback[], format[], ...)` | bool | INSERT ou UPDATE auto |
| `orm_apply_cache(orm_id, row = 0)` | bool | Aplica cache nas vars |
| `orm_addvar_int(orm_id, &var, column[])` | bool | Bind variavel int |
| `orm_addvar_float(orm_id, &Float:var, column[])` | bool | Bind variavel float |
| `orm_addvar_string(orm_id, var[], max_len, column[])` | bool | Bind variavel string |
| `orm_delvar(orm_id, column[])` | bool | Remove binding |
| `orm_clear_vars(orm_id)` | bool | Remove todos os bindings |
| `orm_setkey(orm_id, column[])` | bool | Define chave primaria |

### Codigos ORM

| Constante | Valor | Descricao |
|---|---|---|
| `ORM_OK` | 0 | Sem erro |
| `ORM_NO_DATA` | 1 | SELECT sem resultados |

## Utilidades

| Native | Retorno | Descricao |
|---|---|---|
| `mysql_unprocessed_queries()` | int | Queries pendentes |
| `mysql_log(log_level)` | bool | Configura nivel de log |

### Niveis de log

| Constante | Valor | Descricao |
|---|---|---|
| `MYSQL_LOG_NONE` | 0 | Desativa logs |
| `MYSQL_LOG_ERROR` | 1 | Apenas erros |
| `MYSQL_LOG_WARNING` | 2 | Erros + warnings |
| `MYSQL_LOG_INFO` | 3 | + info |
| `MYSQL_LOG_ALL` | 4 | Tudo (padrao) |

## Forwards

| Forward | Descricao |
|---|---|
| `OnQueryError(errorid, error[], callback[], query[], connId)` | Query threaded falhou |

## Contagem total

| Categoria | Quantidade |
|---|---|
| Conexao | 5 |
| Options | 3 |
| Erro | 2 |
| Queries | 4 |
| Cache | 19 |
| ORM | 15 |
| Utilidades | 2 |
| Forwards | 1 |
| **Total** | **51** |
