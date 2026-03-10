# Tratamento de erros

## mysql_errno

Retorna o codigo do ultimo erro de uma conexao (ou erro global).

```pawn
native mysql_errno(connId = 0);
```

| Parametro | Descricao |
|---|---|
| `connId` | ID da conexao, ou `0` para o ultimo erro global |

### Codigos de erro do plugin

| Constante | Valor | Descricao |
|---|---|---|
| `MYSQL_OK` | 0 | Sem erro |
| `MYSQL_ERROR_CONNECTION_FAILED` | 1 | Falha na conexao |
| `MYSQL_ERROR_INVALID_OPTIONS` | 2 | Handle de options invalido |
| `MYSQL_ERROR_INVALID_CONNECTION` | 3 | Conexao invalida |
| `MYSQL_ERROR_PING_FAILED` | 4 | Ping falhou |
| `MYSQL_ERROR_QUERY_FAILED` | 5 | Falha na query |
| `MYSQL_ERROR_NO_CACHE_ACTIVE` | 6 | Nenhum cache ativo |
| `MYSQL_ERROR_INVALID_ORM` | 7 | ORM invalido |
| `MYSQL_ERROR_ORM_KEY_NOT_SET` | 8 | Chave ORM nao definida |

```pawn
gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp_db");

if (mysql_errno()) {
    printf("Erro de conexao: %d", mysql_errno());
    return 1;
}
```

## mysql_error

Retorna a mensagem de erro em texto.

```pawn
native bool:mysql_error(connId, dest[], max_len = sizeof(dest));
```

```pawn
if (mysql_errno()) {
    new msg[256];
    mysql_error(0, msg);
    printf("Erro: %s", msg);
}
```

## OnQueryError

Forward chamado automaticamente quando uma query threaded falha. E disparado em **todos** os AMX carregados.

```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);
```

| Parametro | Tipo | Descricao |
|---|---|---|
| `errorid` | int | Codigo de erro MySQL (1062, 1045, etc.) ou 0 |
| `error` | string | Mensagem de erro completa |
| `callback` | string | Nome do callback que seria chamado |
| `query` | string | Query SQL que falhou |
| `connId` | int | ID da conexao |

```pawn
public OnQueryError(errorid, const error[], const callback[], const query[], connId) {
    printf("[MySQL Error %d] %s", errorid, error);
    printf("  Callback: %s", callback);
    printf("  Query: %s", query);
    printf("  Connection: %d", connId);
    return 1;
}
```

### Erros comuns do MySQL

| Codigo | Descricao |
|---|---|
| 1045 | Acesso negado (usuario/senha errados) |
| 1049 | Banco de dados desconhecido |
| 1062 | Entrada duplicada (UNIQUE constraint) |
| 1064 | Erro de sintaxe SQL |
| 1146 | Tabela nao existe |
| 1451 | Foreign key constraint falhou |
| 2002 | Nao foi possivel conectar ao servidor |
| 2006 | MySQL server has gone away |

## Logs

### Console

O console do servidor exibe apenas mensagens genericas com codigos de erro:

```
[MySQL] Connection failed (error 1). See logs/mysql.log for details.
[MySQL] Query failed on connection 1 (error 1064). See logs/mysql.log for details.
```

> Nunca exibe queries, senhas ou dados sensiveis no console.

### logs/mysql.log

O arquivo de log contem detalhes completos com timestamp:

```
[2026-02-23 14:30:15] [ERROR] Pool creation failed: Access denied for user 'root'@'localhost'
[2026-02-23 14:30:20] [ERROR] Query error: You have an error in your SQL syntax; ...
[2026-02-23 14:30:30] [WARNING] cache_save failed: maximum saved caches reached (1024).
```

### Niveis de log

```pawn
mysql_log(MYSQL_LOG_NONE);      // Desativa tudo
mysql_log(MYSQL_LOG_ERROR);     // Apenas erros
mysql_log(MYSQL_LOG_WARNING);   // Erros + warnings
mysql_log(MYSQL_LOG_INFO);      // Erros + warnings + info
mysql_log(MYSQL_LOG_ALL);       // Tudo (padrao)
```

## Boas praticas

1. **Sempre verifique `mysql_errno` apos `mysql_connect`** — uma conexao falhada retorna `0`
2. **Implemente `OnQueryError`** — captura erros de queries que voce nao previa
3. **Consulte `logs/mysql.log` para debug** — os detalhes estao la, nao no console
4. **Use `mysql_format` com `%s`** — previne SQL injection automaticamente
5. **Verifique `cache_get_row_count` antes de ler dados** — evita ler cache vazio
