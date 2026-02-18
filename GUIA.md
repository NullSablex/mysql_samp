# mysql_samp - Guia de uso

## Natives

### mysql_options_new

Cria um novo handle de options com valores padrão (porta 3306, SSL desativado).

```pawn
native mysql_options_new();
```

**Retorno:** handle de options.

---

### mysql_options_set_int

Define uma opção numérica.

```pawn
native bool:mysql_options_set_int(handle, option, value);
```

| Opção                    | Valor                        |
|--------------------------|------------------------------|
| `MYSQL_OPT_PORT`         | Porta TCP (padrão: 3306)    |
| `MYSQL_OPT_SSL`          | false = desativado, true = ativado |
| `MYSQL_OPT_CONNECT_TIMEOUT` | Timeout em segundos       |

---

### mysql_options_set_str

Define uma opção de texto.

```pawn
native bool:mysql_options_set_str(handle, option, const value[]);
```

| Opção              | Valor                   |
|--------------------|-------------------------|
| `MYSQL_OPT_SSL_CA` | Caminho do certificado CA |

---

### mysql_connect

Estabelece uma conexão com o servidor MySQL.

```pawn
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
```

| Parâmetro  | Tipo        | Descrição                                                          |
|------------|-------------|--------------------------------------------------------------------|
| `host`     | `string`    | IP, hostname ou caminho do socket Unix (ex: `/var/run/mysqld/mysqld.sock`) |
| `user`     | `string`    | Usuário do banco                                                   |
| `password` | `string`    | Senha do banco                                                     |
| `database` | `string`    | Nome do banco de dados                                             |
| `options`  | `int`       | Handle de options (opcional, padrão usa porta 3306)                |

**Retorno:** ID da conexão (>= 1) em caso de sucesso, ou `0` em caso de falha.

Se `host` começar com `/`, a conexão é feita via socket Unix. Caso contrário, usa TCP.

---

### mysql_close

Fecha uma conexão existente.

```pawn
native bool:mysql_close(connId);
```

**Retorno:** `true` se a conexão foi fechada, `false` se não encontrada.

---

### mysql_status

Retorna informações de status do servidor MySQL.

```pawn
native bool:mysql_status(connId, dest[], max_len = sizeof(dest));
```

| Parâmetro | Descrição                               |
|-----------|-----------------------------------------|
| `connId`  | ID da conexão                           |
| `dest`    | Buffer de destino para a string         |
| `max_len` | Tamanho máximo do buffer                |

**Retorno:** `true` se obteve o status, `false` em caso de falha.

O resultado é uma string compacta com métricas do servidor como `Uptime`, `Threads_connected`, `Questions`, `Slow_queries`, entre outras.

---

### mysql_errno

Retorna o código do último erro.

```pawn
native mysql_errno(connId = 0);
```

| Parâmetro | Descrição                                         |
|-----------|---------------------------------------------------|
| `connId`  | ID da conexão, ou 0 para o último erro global     |

**Códigos de erro:**

| Constante                       | Valor | Descrição                |
|---------------------------------|-------|--------------------------|
| `MYSQL_OK`                      | 0     | Sem erro                 |
| `MYSQL_ERROR_CONNECTION_FAILED` | 1     | Falha na conexão         |
| `MYSQL_ERROR_INVALID_OPTIONS`   | 2     | Handle de options inválido |
| `MYSQL_ERROR_INVALID_CONNECTION`| 3     | Conexão inválida         |
| `MYSQL_ERROR_PING_FAILED`       | 4     | Ping falhou              |
| `MYSQL_ERROR_UNKNOWN`           | 5     | Erro desconhecido        |

Os detalhes do erro são registrados em `logs/mysql.log`.

---

## Exemplos

### Uso básico

Conecta na porta padrão 3306 sem nenhuma configuração extra.

```pawn
#include <a_samp>
#include <mysql_samp>

new gMysql;

public OnGameModeInit() {
    gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp_server");

    if (mysql_errno()) {
        return 1;
    }

    return 1;
}

public OnGameModeExit() {
    mysql_close(gMysql);
    return 1;
}
```

### Uso com options

Options é necessário apenas quando se quer alterar algo além do padrão, como porta, SSL ou timeout.

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
mysql_options_set_int(opts, MYSQL_OPT_SSL, true);
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);

gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp_server", opts);
```
