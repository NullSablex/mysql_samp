# Changelog

Todas as alterações notáveis deste projeto serão documentadas neste arquivo.

## [1.0.0] - 2026-03-09

### Adicionado

#### Options
- `MYSQL_OPT_AUTO_RECONNECT` — option que ativa reconexão automática em caso de queda da conexão com o MySQL durante uma query (padrão: ativado)

#### Queries (non-blocking)
- `mysql_query` — Query threaded com callback e ordenação FIFO (substitui `mysql_tquery` do R41-4)
- `mysql_pquery` — Query paralela sem garantia de ordem (fire-and-forget)
- `mysql_escape_string` — Escape de strings para uso seguro em SQL (função pura, sem conexão)
- `mysql_format` — Formatação printf-like de queries (`%d`, `%f`, `%s`/`%e` com escape automático, `%r` raw)

#### Cache
- `cache_get_row_count` / `cache_get_field_count` — Dimensões do resultado
- `cache_get_field_name` / `cache_get_field_type` — Metadados das colunas
- `cache_get_value_index` / `cache_get_value_index_int` / `cache_get_value_index_float` — Valor por índice
- `cache_get_value_name` / `cache_get_value_name_int` / `cache_get_value_name_float` — Valor por nome
- `cache_is_value_index_null` / `cache_is_value_name_null` — Verificação de NULL
- `cache_affected_rows` / `cache_insert_id` — Metadados de INSERT/UPDATE/DELETE
- `cache_warning_count` — Contagem de warnings do MySQL
- `cache_get_query_exec_time` / `cache_get_query_string` — Debug da query
- `cache_save` / `cache_delete` — Persistência de cache
- `cache_set_active` / `cache_unset_active` — Ativação manual de cache salvo
- `cache_is_any_active` / `cache_is_valid` — Verificação de estado do cache

#### ORM
- `orm_create` / `orm_destroy` — Criação e destruição de instâncias ORM
- `orm_errno` — Código de erro da última operação ORM
- `orm_select` / `orm_update` / `orm_insert` / `orm_delete` — Operações CRUD non-blocking
- `orm_save` — INSERT ou UPDATE automático baseado no valor da chave
- `orm_apply_cache` — Aplica resultado do cache nas variáveis Pawn vinculadas
- `orm_addvar_int` / `orm_addvar_float` / `orm_addvar_string` — Binding de variáveis
- `orm_delvar` / `orm_clear_vars` — Remoção de bindings
- `orm_setkey` — Define coluna de chave primária

#### Callbacks
- `OnQueryError(errorid, error[], callback[], query[], connId)` — Forward chamado quando uma query falha

#### Utilidades
- `mysql_error` — Obtém a mensagem do último erro em texto
- `mysql_set_charset` / `mysql_get_charset` — Configuração de charset da conexão
- `mysql_unprocessed_queries` — Contagem de queries pendentes em execução
- `mysql_log` — Configuração do nível de log em runtime

#### Infraestrutura
- Pool de conexões (`mysql::Pool`) para reutilização segura de conexões em threads
- Sistema de cache com stack (push/pop) para callbacks e armazenamento manual
- QueryManager com threading via `mpsc` e reordenação FIFO
- Callback dispatcher com suporte a parâmetros variádicos (int, float, string)
- Limpeza automática de ORMs quando o AMX é descarregado

#### Testes unitários
- 93 testes cobrindo toda a lógica pura do plugin (sem dependência de MySQL ou SA:MP runtime)
- `error.rs` — Códigos de erro, ErrorState, construtores, clone, igualdade
- `options.rs` — MysqlOptionKind, MysqlOptions defaults, OptionsManager CRUD, wrapping de IDs
- `cache.rs` — CacheEntry (campos, valores, NULL, tipos), CacheManager (stack, save/delete, ativação manual)
- `connection.rs` — escape_string (caracteres especiais, UTF-8, SQL injection), escape_identifier, ConnectionManager
- `query.rs` — QueryManager (ordenação FIFO, dispatch parcial, resultados paralelos, modo misto)
- `orm.rs` — OrmVarBinding, OrmError, OrmManager CRUD, destroy_by_amx, wrapping de IDs

#### Documentação
- Banner no `include/mysql_samp.inc` com nome do projeto, autor, repositório e licença
- `docs/options.md` — documentação completa de todas as options de conexão com exemplos
- `docs/benchmark.md` — benchmark comparativo com o R41-4, incluindo arquivos executáveis (`benchmark/setup.sql`, `benchmark/bench_mysql_samp.pwn`, `benchmark/bench_r41.pwn`)
- `docs/mudancas.md` — Referência de mudanças práticas R41-4 → mysql_samp
- `docs/exemplos-migracao.md` — 14 exemplos de código antes/depois para migração
- `docs/migracao.md` — Guia passo a passo de migração do R41-4
- `docs/api.md` — Referência completa de todas as 51 natives + 1 forward
- `docs/queries.md`, `docs/cache.md`, `docs/orm.md` — Documentação detalhada por módulo
- `docs/erros.md`, `docs/conexao.md`, `docs/seguranca.md`, `docs/instalacao.md`

### Alterado
- `ConnectionEntry` migrado de `mysql::Conn` para `mysql::Pool` (Send+Sync+Clone)
- `MysqlPlugin` agora implementa `on_amx_load`, `on_amx_unload` e `process_tick`
- `enable_process_tick()` habilitado na inicialização do plugin

### Removido
- `MysqlError::Unknown` — variante sem uso removida; variantes seguintes renumeradas (`QueryFailed=5`, `NoCacheActive=6`, `InvalidOrm=7`, `OrmKeyNotSet=8`)
- Limite artificial de 128 queries concorrentes (`MAX_CONCURRENT_QUERIES`) — o `mysql::Pool` já provê backpressure natural; o limite causava rejeição silenciosa de queries acima do teto

### Segurança
- **CWE-89** — `%s` no `mysql_format` agora escapa automaticamente (antes era raw). Novo `%r` para strings cruas
- **CWE-89** — Identificadores SQL no ORM escapados via `escape_identifier()` (remoção de backticks)
- **CWE-787** — Proteção contra escrita fora dos limites no ORM (`max_len` limitado a 4096)
- **CWE-770** — Limite de 1024 caches salvos e 100.000 rows por resultado
- **CWE-252** — Verificação de erros no push de parâmetros para callbacks AMX
- **CWE-190** — Proteção contra overflow de IDs com `wrapping_add` em todos os managers
- **UTF-8 forçado** — `SET NAMES utf8mb4` via `init()` do Pool em todas as conexões (previne ataques multi-byte)
- **Reconexão automática** — queries não falham silenciosamente após queda de conexão; o plugin tenta uma segunda vez em uma conexão fresca antes de reportar erro via `OnQueryError`

## [0.1.0] - 2026-02-22

### Adicionado
- Lançamento inicial: conexão, options, errno, logger com banner
- `mysql_connect` / `mysql_close` / `mysql_status`
- `mysql_options_new` / `mysql_options_set_int` / `mysql_options_set_str`
- `mysql_errno`
- Suporte a Unix socket (detecção por `/` no host)
- TLS via rustls (zero dependências externas)
- Logs detalhados em `logs/mysql.log`
