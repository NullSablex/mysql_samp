# mysql_samp

> Plugin MySQL para SA:MP escrito em Rust — por [NullSablex](https://github.com/NullSablex)

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![SA:MP](https://img.shields.io/badge/SA:MP-0.3.7+-orange)
![Build](https://img.shields.io/badge/build-Linux%20%7C%20Windows-green)
![Architecture](https://img.shields.io/badge/arch-x86%20(32--bit)-lightgrey)
[![Release](https://img.shields.io/github/v/release/NullSablex/MySQL-SAMP?label=download)](https://github.com/NullSablex/mysql_samp/releases/latest)

> [!WARNING]
> Este projeto está em fase inicial de desenvolvimento. A API pode sofrer alterações entre versões.

## Visão geral

**mysql_samp** é um plugin MySQL moderno para SA:MP (San Andreas Multiplayer) construído inteiramente em Rust. Fornece uma API limpa e minimalista para conectividade com banco de dados, sem nenhuma dependência externa em runtime.

### Destaques

- **Zero dependências externas** — sem `libmysqlclient`, sem OpenSSL. O protocolo MySQL e o TLS (via rustls) são compilados diretamente no binário.
- **Deploy simples** — copie o `.so` ou `.dll` e funciona. Sem bibliotecas extras para instalar.
- **Suporte a Unix socket** — conecte via TCP ou socket Unix passando o caminho do socket como host.
- **Seguro por padrão** — o console exibe apenas mensagens genéricas; detalhes sensíveis de erro vão exclusivamente para `logs/mysql.log`.
- **Configuração opcional** — funciona com valores padrão sensíveis (porta 3306). Use a API de options apenas quando precisar de configurações personalizadas.

## Instalação

1. Baixe a versão mais recente para sua plataforma:
   - `mysql_samp.so` (Linux)
   - `mysql_samp.dll` (Windows)
2. Coloque o arquivo no diretório `plugins/` do seu servidor.
3. Copie `include/mysql_samp.inc` para o diretório `pawno/include/`.
4. Adicione ao `server.cfg`:
   ```
   plugins mysql_samp.so
   ```
   No Windows:
   ```
   plugins mysql_samp.dll
   ```

> [!IMPORTANT]
> Não é necessário instalar `libmysqlclient` ou qualquer outra biblioteca. O plugin é auto-contido.

## Início rápido

```pawn
#include <a_samp>
#include <mysql_samp>

new gMysql;

public OnGameModeInit() {
    gMysql = mysql_connect("127.0.0.1", "root", "senha", "samp_db");

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

## Referência da API

### Conexão

| Native | Descrição |
|---|---|
| `mysql_connect(host[], user[], pass[], db[], options = 0)` | Conecta ao servidor MySQL. Retorna ID da conexão ou `0` em caso de falha. |
| `mysql_close(connId)` | Fecha uma conexão existente. |
| `mysql_status(connId, dest[], max_len)` | Obtém métricas de status do servidor. |

### Options

Options são **totalmente opcionais**. Use apenas quando precisar de configurações diferentes do padrão.

| Native | Descrição |
|---|---|
| `mysql_options_new()` | Cria um handle de options com valores padrão. |
| `mysql_options_set_int(handle, option, value)` | Define uma opção numérica. |
| `mysql_options_set_str(handle, option, value[])` | Define uma opção de texto. |

| Opção | Tipo | Descrição |
|---|---|---|
| `MYSQL_OPT_PORT` | int | Porta TCP (padrão: 3306) |
| `MYSQL_OPT_SSL` | int | Ativar SSL (`true` / `false`) |
| `MYSQL_OPT_SSL_CA` | string | Caminho do certificado CA |
| `MYSQL_OPT_CONNECT_TIMEOUT` | int | Timeout de conexão em segundos |

### Tratamento de erros

| Native | Descrição |
|---|---|
| `mysql_errno(connId = 0)` | Retorna o código do último erro. `0` = sem erro. |

> [!NOTE]
> Detalhes dos erros são registrados em `logs/mysql.log` com timestamp. O console do servidor exibe apenas mensagens genéricas com códigos de erro por segurança.

## Compilando o código-fonte

### Requisitos

- Toolchain Rust com targets: `i686-unknown-linux-gnu`, `i686-pc-windows-gnu`
- Nenhuma biblioteca do sistema necessária (build 100% Rust)

### Build de desenvolvimento

```bash
cargo build
```

### Build de release (Linux + Windows)

```bash
bash scripts/build.sh
```

Os arquivos são gerados em `dist/` com checksums SHA-256.

> [!CAUTION]
> Este plugin é distribuído sob a GPL v3. Qualquer trabalho derivado deve manter o código-fonte aberto sob a mesma licença.

## Licença

Copyright (c) 2026 NullSablex

Este projeto está licenciado sob a [GNU General Public License v3.0](LICENSE).
