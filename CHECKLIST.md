# Checklist: mysql_samp vs R41-4

## Conexão

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `mysql_connect` | Sim | Sim | Nosso não usa tags |
| `mysql_connect_file` | Sim | - | Conexão via .ini |
| `mysql_close` | Sim | Sim | |
| `mysql_errno` | Sim | Sim | Nosso retorna código interno por ora, nativo do MySQL quando queries forem implementadas |
| `mysql_error` | Sim | - | Retorna string do erro |
| `mysql_escape_string` | Sim | - | Escape de strings para queries |
| `mysql_format` | Sim | - | printf-like para queries |
| `mysql_set_charset` | Sim | - | |
| `mysql_get_charset` | Sim | - | |
| `mysql_stat` / `mysql_status` | Sim | Sim | Nosso usa `mysql_status` |
| `mysql_unprocessed_queries` | Sim | - | Queries pendentes na fila |
| `mysql_log` | Sim | - | Nível de log configurável |
| Socket Unix | - | Sim | Detecta por `/` no host |

## Options

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| Criar options | `mysql_init_options` | `mysql_options_new` | |
| Definir opção | `mysql_set_option` (variadic) | `mysql_options_set_int` / `_set_str` | Nosso separa int e str |
| `mysql_global_options` | Sim | - | Options globais (duplicatas) |
| AUTO_RECONNECT | Sim | - | |
| MULTI_STATEMENTS | Sim | - | |
| POOL_SIZE | Sim | - | SA:MP é single-threaded, sem uso |
| SERVER_PORT | Sim | Sim | `MYSQL_OPT_PORT` |
| SSL_ENABLE | Sim | Sim | `MYSQL_OPT_SSL` |
| SSL_KEY_FILE | Sim | - | |
| SSL_CERT_FILE | Sim | - | |
| SSL_CA_FILE | Sim | Sim | `MYSQL_OPT_SSL_CA` |
| SSL_CA_PATH | Sim | - | |
| SSL_CIPHER | Sim | - | |
| CONNECT_TIMEOUT | - | Sim | Exclusivo nosso |

## Queries

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `mysql_query` | Sim | - | Query síncrona com cache |
| `mysql_tquery` | Sim | - | Query threaded com callback |
| `mysql_pquery` | Sim | - | Query paralela (sem garantia de ordem) |
| `mysql_query_file` | Sim | - | Query de arquivo SQL |
| `mysql_tquery_file` | Sim | - | Query threaded de arquivo |

## Cache

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `cache_get_row_count` | Sim | - | |
| `cache_get_field_count` | Sim | - | |
| `cache_get_result_count` | Sim | - | |
| `cache_get_field_name` | Sim | - | |
| `cache_get_field_type` | Sim | - | |
| `cache_set_result` | Sim | - | |
| `cache_get_value_index` | Sim | - | String por índice |
| `cache_get_value_index_int` | Sim | - | Int por índice |
| `cache_get_value_index_float` | Sim | - | Float por índice |
| `cache_is_value_index_null` | Sim | - | |
| `cache_get_value_name` | Sim | - | String por nome |
| `cache_get_value_name_int` | Sim | - | Int por nome |
| `cache_get_value_name_float` | Sim | - | Float por nome |
| `cache_is_value_name_null` | Sim | - | |
| `cache_save` | Sim | - | Salva cache para uso posterior |
| `cache_delete` | Sim | - | |
| `cache_set_active` | Sim | - | |
| `cache_unset_active` | Sim | - | |
| `cache_is_any_active` | Sim | - | |
| `cache_is_valid` | Sim | - | |
| `cache_affected_rows` | Sim | - | |
| `cache_insert_id` | Sim | - | |
| `cache_warning_count` | Sim | - | |
| `cache_get_query_exec_time` | Sim | - | |
| `cache_get_query_string` | Sim | - | |

## ORM

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `orm_create` | Sim | - | |
| `orm_destroy` | Sim | - | |
| `orm_errno` | Sim | - | |
| `orm_apply_cache` | Sim | - | |
| `orm_select` / `orm_load` | Sim | - | |
| `orm_update` | Sim | - | |
| `orm_insert` | Sim | - | |
| `orm_delete` | Sim | - | |
| `orm_save` | Sim | - | |
| `orm_addvar_int` | Sim | - | |
| `orm_addvar_float` | Sim | - | |
| `orm_addvar_string` | Sim | - | |
| `orm_clear_vars` | Sim | - | |
| `orm_delvar` | Sim | - | |
| `orm_setkey` | Sim | - | |

## Callbacks

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `OnQueryError` | Sim | - | Forward chamado em erro de query |

## Extras (exclusivo mysql_samp)

| Funcionalidade | Notas |
|---|---|
| Zero dependências externas | Sem libmysqlclient, sem OpenSSL |
| TLS via rustls | Embutido no binário |
| `MYSQL_OPT_CONNECT_TIMEOUT` | Timeout de conexão configurável |
| Logs detalhados em arquivo | `logs/mysql.log` com timestamp |
| Banner informativo | Data/hora de build |
