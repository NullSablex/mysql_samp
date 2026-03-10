# Conexao

## mysql_connect

Estabelece uma conexao com o servidor MySQL.

```pawn
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
```

| Parametro | Tipo | Descricao |
|---|---|---|
| `host` | string | IP, hostname ou caminho do socket Unix |
| `user` | string | Usuario do banco |
| `password` | string | Senha do banco |
| `database` | string | Nome do banco de dados |
| `options` | int | Handle de options (opcional, 0 = padrao) |

**Retorno:** ID da conexao (>= 1) em caso de sucesso, ou `0` em caso de falha.

### Conexao basica (porta 3306)

```pawn
new gMysql;

public OnGameModeInit() {
    gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp_db");

    if (mysql_errno()) {
        printf("Falha ao conectar: erro %d", mysql_errno());
        return 1;
    }

    printf("Conectado! ID: %d", gMysql);
    return 1;
}
```

### Conexao via Unix socket

Se o `host` comecar com `/`, a conexao e feita via socket Unix:

```pawn
gMysql = mysql_connect("/var/run/mysqld/mysqld.sock", "root", "", "samp_db");
```

### Multiplas conexoes

Voce pode manter varias conexoes simultaneas:

```pawn
new gMain, gLogs;

public OnGameModeInit() {
    gMain = mysql_connect("127.0.0.1", "root", "senha", "samp_main");
    gLogs = mysql_connect("127.0.0.1", "root", "senha", "samp_logs");
    return 1;
}
```

## mysql_close

Fecha uma conexao existente e libera os recursos do pool.

```pawn
native bool:mysql_close(connId);
```

**Retorno:** `true` se a conexao foi fechada, `false` se nao encontrada.

```pawn
public OnGameModeExit() {
    mysql_close(gMysql);
    return 1;
}
```

## mysql_status

Retorna metricas de status do servidor MySQL.

```pawn
native bool:mysql_status(connId, dest[], max_len = sizeof(dest));
```

**Retorno:** `true` se obteve o status, `false` em caso de falha.

O resultado e uma string com metricas como `Uptime`, `Threads_connected`, `Questions`, `Slow_queries`, `Opens`, `Flush_tables`, `Open_tables`, `Queries_per_second_avg`.

```pawn
new status[256];
mysql_status(gMysql, status);
printf("Status: %s", status);
// Output: Uptime: 12345  Threads_connected: 3  Questions: 567  ...
```

## Options

Options sao **totalmente opcionais**. Use apenas quando precisar de configuracoes diferentes do padrao.

### mysql_options_new

Cria um handle de options com valores padrao.

```pawn
native mysql_options_new();
```

**Retorno:** handle de options (>= 1).

### mysql_options_set_int

Define uma opcao numerica.

```pawn
native bool:mysql_options_set_int(handle, option, value);
```

### mysql_options_set_str

Define uma opcao de texto.

```pawn
native bool:mysql_options_set_str(handle, option, const value[]);
```

### Opcoes disponiveis

| Opcao | Tipo | Padrao | Descricao |
|---|---|---|---|
| `MYSQL_OPT_PORT` | int | 3306 | Porta TCP |
| `MYSQL_OPT_SSL` | int | false | Ativar TLS (via rustls) |
| `MYSQL_OPT_SSL_CA` | string | — | Caminho do certificado CA |
| `MYSQL_OPT_CONNECT_TIMEOUT` | int | — | Timeout de conexao em segundos |

### Exemplo com options

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
mysql_options_set_int(opts, MYSQL_OPT_SSL, true);
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);

gMysql = mysql_connect("db.example.com", "user", "pass", "samp_db", opts);
```

## Charset

O plugin forca `utf8mb4` como charset padrao em todas as conexoes. Voce pode alterar se necessario.

### mysql_set_charset

```pawn
native bool:mysql_set_charset(connId, const charset[]);
```

Executa `SET NAMES` com o charset especificado.

```pawn
mysql_set_charset(gMysql, "latin1");
```

### mysql_get_charset

```pawn
native bool:mysql_get_charset(connId, dest[], max_len = sizeof(dest));
```

Retorna o charset atual da conexao.

```pawn
new charset[32];
mysql_get_charset(gMysql, charset);
printf("Charset: %s", charset); // utf8mb4
```

## Pool de conexoes

Internamente, cada `mysql_connect` cria um `mysql::Pool`. Isso significa que:

- Conexoes sao reutilizadas automaticamente entre queries
- Threads de query obtem conexoes do pool sem conflito
- O pool e thread-safe (Send+Sync+Clone)
- Nao e necessario gerenciar conexoes manualmente
