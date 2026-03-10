# Seguranca

O mysql_samp foi desenvolvido com seguranca como prioridade. Este documento descreve as protecoes implementadas e boas praticas de uso.

## SQL Injection

### Protecao automatica

O plugin protege contra SQL injection em multiplas camadas:

1. **`mysql_format` com `%s`** — Escapa automaticamente a string (aspas, barras, null bytes, etc.)
2. **`mysql_escape_string`** — Funcao pura para escape manual
3. **ORM** — Strings vinculadas sao escapadas automaticamente no SQL gerado
4. **Identificadores** — Nomes de tabela/coluna no ORM sao sanitizados via `escape_identifier`

### Exemplo seguro

```pawn
// SEGURO: %s escapa automaticamente
new query[256];
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM players WHERE name = '%s'", playerName);
```

### Exemplo inseguro

```pawn
// INSEGURO: concatenacao direta — NUNCA faca isso
new query[256];
format(query, sizeof(query),
    "SELECT * FROM players WHERE name = '%s'", playerName);
// Se playerName = "'; DROP TABLE players; --" → SQL injection!
```

### %s vs %r

| Especificador | Escape | Quando usar |
|---|---|---|
| `%s` | Sim | Input do usuario, dados nao confiaveis |
| `%r` | Nao | Valores internos confiaveis (nomes de tabela, constantes) |

> **Regra:** Use `%s` para tudo. Use `%r` apenas quando voce tem certeza absoluta de que o valor e seguro.

## UTF-8

O plugin forca `SET NAMES utf8mb4` em **todas** as conexoes do pool via `init()` do `OptsBuilder`. Isso previne:

- **Ataques multi-byte** (GBK injection) onde caracteres multi-byte podem "engolir" barras de escape
- **Truncamento de dados** em caracteres especiais
- **Inconsistencia de encoding** entre cliente e servidor

Voce pode alterar o charset com `mysql_set_charset`, mas o padrao UTF-8 e o mais seguro.

## Limites de recursos

O plugin implementa limites para prevenir esgotamento de memoria e threads:

| Recurso | Limite | Protecao |
|---|---|---|
| Caches salvos | 1024 | Previne memory exhaustion (CWE-770) |
| Rows por resultado | 100.000 | Previne alocacao massiva de memoria (CWE-770) |
| ORM string max_len | 4096 | Previne escrita fora dos limites (CWE-787) |

Quando um limite e atingido:
- A operacao falha graciosamente (retorna false/0)
- Um warning e registrado em `logs/mysql.log`
- O servidor continua funcionando normalmente

## Integer overflow

Todos os contadores de ID internos (conexoes, options, ORM, cache) usam `wrapping_add` com validacao de minimo 1. Isso previne:

- Overflow aritmetico em operacoes longas (CWE-190)
- IDs negativos ou zero acidentais

## Callback safety

O dispatch de callbacks verifica erros em cada operacao:
- Se um `push` de parametro na stack AMX falhar, o callback e abortado
- Um erro e registrado indicando qual callback falhou
- O servidor continua funcionando (nao crasheia)

## Logs seguros

| Destino | Conteudo |
|---|---|
| Console | Mensagens genericas com codigos de erro |
| `logs/mysql.log` | Detalhes completos com timestamp |

O console **nunca** exibe:
- Queries SQL completas
- Senhas ou credenciais
- Dados de resultado

## Boas praticas

### 1. Sempre use `mysql_format` com `%s`

```pawn
// Correto
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM users WHERE name = '%s'", input);

// Errado — vulneravel a SQL injection
format(query, sizeof(query),
    "SELECT * FROM users WHERE name = '%s'", input);
```

### 2. Implemente OnQueryError

```pawn
public OnQueryError(errorid, const error[], const callback[], const query[], connId) {
    printf("[MySQL] Error %d: %s", errorid, error);
    return 1;
}
```

### 3. Valide dados antes de usar

```pawn
forward OnData();
public OnData() {
    if (cache_get_row_count() <= 0) return;

    // Agora e seguro ler dados
    new val = cache_get_value_name_int(0, "id");
}
```

### 4. Libere recursos

```pawn
// Destrua ORMs quando nao precisar mais
orm_destroy(ormId);

// Delete caches salvos quando nao precisar mais
cache_delete(cacheId);

// Feche conexoes ao descarregar
mysql_close(connId);
```

### 5. Nao confie em %r

```pawn
// NUNCA faca isso com input do usuario
mysql_format(gMysql, query, sizeof(query),
    "SELECT * FROM %r WHERE id = %d", userInput, id);
// userInput poderia ser "players; DROP TABLE users; --"
```

## CVEs/CWEs mitigados

| CWE | Severidade | Mitigacao |
|---|---|---|
| CWE-89 | Alta | Escape automatico em %s, %e, ORM strings e identificadores |
| CWE-787 | Alta | Limite de max_len a 4096 no ORM |
| CWE-770 | Media | Limites em caches salvos e rows por resultado |
| CWE-252 | Media | Verificacao de erros no push de parametros AMX |
| CWE-190 | Baixa | wrapping_add em todos os contadores de ID |
