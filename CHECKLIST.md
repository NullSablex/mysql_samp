# Checklist: mysql_samp vs R41-4

## Conexão

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `mysql_connect` | Sim | Sim | Nosso não usa tags |
| `mysql_connect_file` | Sim | - | Conexão via .ini |
| `mysql_close` | Sim | Sim | |
| `mysql_errno` | Sim | Sim | Retorna código interno do plugin |
| `mysql_error` | Sim | - | Retorna string do erro |
| `mysql_escape_string` | Sim | Sim | Escape puro (sem connId) |
| `mysql_format` | Sim | Sim | printf-like com `%d`, `%f`, `%s`, `%e` |
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
| POOL_SIZE | Sim | - | Nosso usa Pool interno (2 conns) |
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
| `mysql_query` | Sim (sync) | Sim (non-blocking FIFO) | Nosso é sempre threaded, substitui tquery |
| `mysql_tquery` | Sim | - | Substituído por `mysql_query` (non-blocking) |
| `mysql_pquery` | Sim | Sim | Query paralela sem ordem |
| `mysql_query_file` | Sim | - | Query de arquivo SQL |
| `mysql_tquery_file` | Sim | - | Query threaded de arquivo |

## Cache

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `cache_get_row_count` | Sim | Sim | |
| `cache_get_field_count` | Sim | Sim | |
| `cache_get_result_count` | Sim | - | Multi-result sets (desnecessário) |
| `cache_get_field_name` | Sim | Sim | |
| `cache_get_field_type` | Sim | - | |
| `cache_set_result` | Sim | - | Multi-result sets (desnecessário) |
| `cache_get_value_index` | Sim | Sim | String por índice |
| `cache_get_value_index_int` | Sim | Sim | Int por índice |
| `cache_get_value_index_float` | Sim | Sim | Float por índice |
| `cache_is_value_index_null` | Sim | Sim | |
| `cache_get_value_name` | Sim | Sim | String por nome |
| `cache_get_value_name_int` | Sim | Sim | Int por nome |
| `cache_get_value_name_float` | Sim | Sim | Float por nome |
| `cache_is_value_name_null` | Sim | Sim | |
| `cache_save` | Sim | Sim | Salva cache para uso posterior |
| `cache_delete` | Sim | Sim | |
| `cache_set_active` | Sim | Sim | |
| `cache_unset_active` | Sim | Sim | |
| `cache_is_any_active` | Sim | - | Disponível internamente |
| `cache_is_valid` | Sim | - | Disponível internamente |
| `cache_affected_rows` | Sim | Sim | |
| `cache_insert_id` | Sim | Sim | |
| `cache_warning_count` | Sim | - | Raramente usado |
| `cache_get_query_exec_time` | Sim | Sim | |
| `cache_get_query_string` | Sim | Sim | |

## ORM

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `orm_create` | Sim | Sim | |
| `orm_destroy` | Sim | Sim | |
| `orm_errno` | Sim | Sim | |
| `orm_apply_cache` | Sim | Sim | |
| `orm_select` / `orm_load` | Sim | Sim | Non-blocking |
| `orm_update` | Sim | Sim | Non-blocking |
| `orm_insert` | Sim | Sim | Non-blocking |
| `orm_delete` | Sim | Sim | Non-blocking |
| `orm_save` | Sim | Sim | INSERT se key=0, UPDATE caso contrário |
| `orm_addvar_int` | Sim | Sim | |
| `orm_addvar_float` | Sim | Sim | |
| `orm_addvar_string` | Sim | Sim | |
| `orm_clear_vars` | Sim | Sim | |
| `orm_delvar` | Sim | Sim | |
| `orm_setkey` | Sim | Sim | |

## Callbacks

| Funcionalidade | R41-4 | mysql_samp | Notas |
|---|---|---|---|
| `OnQueryError` | Sim | Sim | Forward chamado em erro de query |

## Extras (exclusivo mysql_samp)

| Funcionalidade | Notas |
|---|---|
| Zero dependências externas | Sem libmysqlclient, sem OpenSSL |
| TLS via rustls | Embutido no binário |
| `MYSQL_OPT_CONNECT_TIMEOUT` | Timeout de conexão configurável |
| Logs detalhados em arquivo | `logs/mysql.log` com timestamp |
| Banner informativo | Data/hora de build |
| Pool de conexões | `mysql::Pool` para threading seguro |
| Queries 100% non-blocking | Sem bloqueio do servidor |
| Limpeza automática de ORM | ORMs destruídos quando AMX descarrega |
