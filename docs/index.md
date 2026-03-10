# Documentação — mysql_samp

Plugin MySQL para SA:MP e open.mp escrito em Rust. Zero dependências externas, queries non-blocking, cache e ORM integrados.

---

## Por onde começar?

<div class="grid cards" markdown>

-   :material-download: **Novo por aqui**

    ---

    Instale o plugin e configure sua primeira conexão.

    [Instalação](instalacao.md) · [Conexão](conexao.md)

-   :material-swap-horizontal: **Migrando do R41-4**

    ---

    Veja o que mudou e adapte seu código existente.

    [Guia de migração](migracao.md) · [O que mudou](mudancas.md)

-   :material-book-open-variant: **Referência**

    ---

    Tabela completa de natives, forwards e enums.

    [API completa](api.md)

-   :material-speedometer: **Benchmark**

    ---

    Resultados de desempenho em cenários reais.

    [Ver benchmark](benchmark.md)

</div>

---

## Exemplo mínimo

Conexão, query assíncrona e leitura do resultado no callback:

```pawn
#include <mysql_samp>

new MySQL:g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "senha", "banco");

    // Query FIFO (non-blocking) — executa em thread separada
    mysql_query(g_mysql, "SELECT id, nome FROM jogadores LIMIT 5", "OnJogadoresCarregados", "");
    return 1;
}

forward OnJogadoresCarregados(errorid, error[]);
public OnJogadoresCarregados(errorid, error[])
{
    if (errorid != 0) {
        printf("[MySQL] Erro %d: %s", errorid, error);
        return;
    }

    while (cache_next_row()) {
        new id, nome[MAX_PLAYER_NAME];
        cache_get_value_name_int("id", id);
        cache_get_value_name("nome", nome);
        printf("Jogador #%d: %s", id, nome);
    }
}

public OnGameModeExit()
{
    mysql_close(g_mysql);
    return 1;
}
```

---

## Tópicos

| Tópico | Descrição |
|---|---|
| [Queries](queries.md) | `mysql_query`, `mysql_pquery`, `mysql_format`, escape de strings |
| [Cache](cache.md) | Leitura de resultados, navegação entre linhas, cache salvo |
| [ORM](orm.md) | Mapeamento de variáveis Pawn para colunas, CRUD automático |
| [Options](options.md) | Configuração de porta, charset, timeout e outros |
| [Segurança](seguranca.md) | Proteção contra SQL injection, limites e boas práticas |
| [Erros](erros.md) | `mysql_errno`, `OnQueryError`, códigos de erro do MySQL |
