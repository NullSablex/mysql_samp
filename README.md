# mysql_samp

> Plugin MySQL para SA:MP escrito em Rust — por [NullSablex](https://github.com/NullSablex)

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![SA:MP](https://img.shields.io/badge/SA:MP-0.3.7+-orange)
![open.mp](https://img.shields.io/badge/open.mp-compatível-orange)
![Build](https://img.shields.io/badge/build-Linux%20%7C%20Windows-green)
![Architecture](https://img.shields.io/badge/arch-x86%20(32--bit)-lightgrey)
[![Release](https://img.shields.io/github/v/release/NullSablex/MySQL-SAMP?label=download)](https://github.com/NullSablex/mysql_samp/releases/latest)

> [!WARNING]
> Este projeto está em fase inicial de desenvolvimento. A API pode sofrer alterações entre versões.

## Visão geral

**mysql_samp** é um plugin MySQL moderno para SA:MP (San Andreas Multiplayer) e [open.mp](https://open.mp) construído inteiramente em Rust. Fornece uma API completa para conectividade com banco de dados, queries non-blocking, sistema de cache e ORM, sem nenhuma dependência externa em runtime.

### Destaques

- **Zero dependências externas** — sem `libmysqlclient`, sem OpenSSL. O protocolo MySQL e o TLS (via rustls) são compilados diretamente no binário.
- **Todas as queries são non-blocking** — `mysql_query` executa em threads separadas com ordenação FIFO. Nunca trava o servidor.
- **Pool de conexões** — reutilização automática de conexões via `mysql::Pool`, com thread safety nativa.
- **ORM integrado** — mapeamento de variáveis Pawn para colunas do banco com operações CRUD automáticas.
- **Sistema de cache** — acesso aos resultados via stack automático ou persistência manual com `cache_save`.
- **Seguro por padrão** — escape automático de strings, UTF-8 forçado, proteção contra SQL injection e memory exhaustion.
- **Deploy simples** — copie o `.so` ou `.dll` e funciona. Sem bibliotecas extras para instalar.

## Instalação

1. Baixe a versão mais recente para sua plataforma:
   - `mysql_samp.so` (Linux)
   - `mysql_samp.dll` (Windows)
2. Coloque o arquivo no diretório `plugins/` do seu servidor.
3. Copie `mysql_samp.inc` para a pasta de includes do compilador:
   - **Windows:** `pawno/include/` ou `qawno/include/`
   - **Linux:** `include/` (na raiz do servidor)
4. Adicione ao `server.cfg` (ou `config.json` no open.mp):
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

    // Query non-blocking com callback
    mysql_query(gMysql, "SELECT * FROM players LIMIT 10", "OnPlayersLoaded");
    return 1;
}

forward OnPlayersLoaded();
public OnPlayersLoaded() {
    new rows = cache_get_row_count();
    printf("Jogadores encontrados: %d", rows);

    new name[MAX_PLAYER_NAME];
    for (new i = 0; i < rows; i++) {
        cache_get_value_name(i, "name", name);
        printf("  - %s", name);
    }
}

public OnGameModeExit() {
    mysql_close(gMysql);
    return 1;
}
```

## Documentacao

A documentacao completa do plugin esta em [docs/](docs/):

| Documento | Conteudo |
|---|---|
| [Instalacao e configuracao](docs/instalacao.md) | Setup, server.cfg, requisitos |
| [Conexao](docs/conexao.md) | mysql_connect, mysql_close, options, charset, SSL |
| [Queries](docs/queries.md) | mysql_query, mysql_pquery, mysql_format, mysql_escape_string |
| [Cache](docs/cache.md) | Todas as funcoes cache_*, save/restore, ciclo de vida |
| [ORM](docs/orm.md) | Mapeamento objeto-relacional, CRUD, bindings |
| [Tratamento de erros](docs/erros.md) | mysql_errno, mysql_error, OnQueryError, codigos |
| [Referencia da API](docs/api.md) | Tabela completa de todas as natives e forwards |
| [Seguranca](docs/seguranca.md) | Escape, UTF-8, limites, boas praticas |
| [Migracao do R41-4](docs/migracao.md) | Diferencas e como migrar do mysql R41-4 |

## Compilando o codigo-fonte

### Requisitos

- Toolchain Rust com targets: `i686-unknown-linux-gnu`, `i686-pc-windows-gnu`
- Nenhuma biblioteca do sistema necessaria (build 100% Rust)

### Build de desenvolvimento

```bash
cargo build
```

### Build de release (Linux + Windows)

```bash
bash scripts/build.sh
```

Os arquivos sao gerados em `dist/` com checksums SHA-256.

> [!CAUTION]
> Este plugin e distribuido sob a GPL v3. Qualquer trabalho derivado deve manter o codigo-fonte aberto sob a mesma licenca.

## Licenca

Copyright (c) 2026 NullSablex

Este projeto esta licenciado sob a [GNU General Public License v3.0](LICENSE).
