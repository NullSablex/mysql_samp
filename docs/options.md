# Options de conexão

O mysql_samp permite configurar parâmetros opcionais de conexão antes de chamar `mysql_connect`. Sem options, os valores padrão são usados automaticamente.

---

## Como usar

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
mysql_options_set_int(opts, MYSQL_OPT_SSL, 1);
mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "/etc/ssl/certs/ca.pem");

new gMysql = mysql_connect("127.0.0.1", "root", "senha", "meu_banco", opts);
```

O handle de options pode ser reutilizado em múltiplas conexões. Ele não é destruído automaticamente — você pode descartá-lo após o `mysql_connect` sem efeitos colaterais.

---

## Natives

```pawn
native mysql_options_new();
native bool:mysql_options_set_int(handle, option, value);
native bool:mysql_options_set_str(handle, option, const value[]);
```

- `mysql_options_new()` — cria um novo conjunto de options com valores padrão. Retorna o handle (>= 1) ou 0 em caso de falha.
- `mysql_options_set_int(handle, option, value)` — define uma option de valor inteiro. Retorna `true` em sucesso.
- `mysql_options_set_str(handle, option, value[])` — define uma option de valor string. Retorna `true` em sucesso.

---

## Options disponíveis

### `MYSQL_OPT_PORT` (int)

**Padrão:** `3306`

Porta TCP do servidor MySQL.

```pawn
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
```

Use quando o MySQL não está na porta padrão, ou quando você usa um proxy/túnel em outra porta.

---

### `MYSQL_OPT_SSL` (int, bool)

**Padrão:** `0` (desativado)

Ativa ou desativa o uso de SSL/TLS na conexão.

```pawn
mysql_options_set_int(opts, MYSQL_OPT_SSL, 1); // ativar
mysql_options_set_int(opts, MYSQL_OPT_SSL, 0); // desativar
```

O mysql_samp usa **rustls** internamente — sem dependência de `libssl` do sistema. Funciona em qualquer distribuição Linux sem instalar nada.

> **Atenção:** ativar SSL sem fornecer `MYSQL_OPT_SSL_CA` faz a conexão aceitar qualquer certificado do servidor. Para validação completa, forneça o caminho do CA via `MYSQL_OPT_SSL_CA`.

---

### `MYSQL_OPT_SSL_CA` (string)

**Padrão:** nenhum (SSL sem verificação de CA)

Caminho absoluto para o arquivo de certificado CA (`.pem`) usado para validar o certificado do servidor MySQL.

```pawn
mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "/etc/ssl/certs/mysql-ca.pem");
```

Requer que `MYSQL_OPT_SSL` esteja ativado. Garante que a conexão seja estabelecida apenas com o servidor correto, prevenindo ataques man-in-the-middle.

> Aceita somente valores via `mysql_options_set_str`. Chamar `mysql_options_set_int` com esta option retorna `false`.

---

### `MYSQL_OPT_CONNECT_TIMEOUT` (int, segundos)

**Padrão:** sem timeout (aguarda indefinidamente)

Tempo máximo em segundos para estabelecer a conexão TCP inicial com o servidor MySQL.

```pawn
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);
```

Se o servidor não responder dentro do prazo, `mysql_connect` retorna 0 e o erro fica disponível via `mysql_errno` / `mysql_error`.

Recomendado para ambientes de produção onde falhas de rede devem ser detectadas rapidamente em vez de travar o servidor por tempo indeterminado na inicialização.

---

### `MYSQL_OPT_AUTO_RECONNECT` (int, bool)

**Padrão:** `1` (ativado)

Controla a reconexão automática em caso de queda da conexão com o MySQL.

```pawn
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 1); // ativar (padrão)
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 0); // desativar
```

**Comportamento quando ativado:** se uma query falhar por perda de conexão — por exemplo, o MySQL reiniciou, o `wait_timeout` expirou, ou houve uma interrupção de rede — o plugin tenta automaticamente obter uma nova conexão do pool e reexecutar a query uma vez antes de reportar o erro ao callback `OnQueryError`.

**Comportamento quando desativado:** qualquer falha de conexão durante uma query é reportada imediatamente via `OnQueryError`, sem tentativa de reconexão.

**Quando desativar:** se o seu gamemode precisa saber exatamente quando uma reconexão ocorreu (por exemplo, para reiniciar transações ou tomar ação específica em caso de queda), desative e trate o erro manualmente em `OnQueryError`.

> Esta option afeta apenas queries em andamento (via `mysql_query` / `mysql_pquery`). A reconexão inicial ao banco na chamada de `mysql_connect` é controlada pelo `MYSQL_OPT_CONNECT_TIMEOUT`.

---

## Tabela resumo

| Constant | Tipo | Padrão | Nativa |
|---|---|---|---|
| `MYSQL_OPT_PORT` | int | `3306` | `set_int` |
| `MYSQL_OPT_SSL` | int (bool) | `0` | `set_int` |
| `MYSQL_OPT_SSL_CA` | string | — | `set_str` |
| `MYSQL_OPT_CONNECT_TIMEOUT` | int (segundos) | sem timeout | `set_int` |
| `MYSQL_OPT_AUTO_RECONNECT` | int (bool) | `1` | `set_int` |

---

## Exemplos completos

### Conexão básica sem options

```pawn
new gMysql = mysql_connect("127.0.0.1", "root", "senha", "meu_banco");
```

Porta 3306, sem SSL, sem timeout, reconexão automática ativada.

---

### Porta customizada

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);

new gMysql = mysql_connect("127.0.0.1", "root", "senha", "meu_banco", opts);
```

---

### SSL com CA + timeout

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3306);
mysql_options_set_int(opts, MYSQL_OPT_SSL, 1);
mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "/etc/ssl/certs/mysql-ca.pem");
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 15);

new gMysql = mysql_connect("db.servidor.com", "usuario", "senha", "producao", opts);
if (!gMysql)
{
    new err[256];
    mysql_error(0, err);
    printf("[MySQL] Falha na conexao: %s", err);
}
```

---

### Desativar reconexão automática (controle manual)

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 0);

new gMysql = mysql_connect("127.0.0.1", "root", "senha", "meu_banco", opts);
```

```pawn
// Tratamento manual de erros de conexão
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    if (errorid == MYSQL_ERROR_QUERY_FAILED)
    {
        printf("[MySQL] Query falhou na conexao %d: %s", connId, error);
        // Lógica de recuperação personalizada aqui
    }
}
```
