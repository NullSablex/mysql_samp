# Queries

Todas as queries no mysql_samp sao **non-blocking**. Elas executam em threads separadas e o resultado e entregue via callback no tick seguinte. O servidor nunca trava esperando o banco.

## mysql_query

Query threaded com callback e ordenacao FIFO. Os callbacks sao chamados na **mesma ordem** em que as queries foram submetidas.

```pawn
native bool:mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
```

| Parametro | Tipo | Descricao |
|---|---|---|
| `connId` | int | ID da conexao |
| `query` | string | Query SQL |
| `callback` | string | Nome da public a ser chamada (opcional) |
| `format` | string | Formato dos parametros extras: `d` ou `i`=int, `f`=float, `s`=string |
| `...` | variadic | Parametros extras passados ao callback |

**Retorno:** `true` se a query foi submetida, `false` em caso de erro.

### Exemplo basico

```pawn
mysql_query(gMysql, "SELECT * FROM players WHERE level > 5", "OnHighLevelPlayers");

forward OnHighLevelPlayers();
public OnHighLevelPlayers() {
    new rows = cache_get_row_count();
    printf("Jogadores nivel alto: %d", rows);
}
```

### Passando parametros ao callback

```pawn
mysql_query(gMysql, query, "OnPlayerLoaded", "d", playerid);

forward OnPlayerLoaded(playerid);
public OnPlayerLoaded(playerid) {
    if (cache_get_row_count() > 0) {
        new name[MAX_PLAYER_NAME];
        cache_get_value_name(0, "name", name);
        printf("Jogador %d: %s", playerid, name);
    }
}
```

### Query sem callback (fire-and-forget)

```pawn
mysql_query(gMysql, "UPDATE players SET last_login = NOW() WHERE id = 1");
```

## mysql_pquery

Query paralela sem garantia de ordem. Os callbacks sao chamados assim que cada query termina, independente da ordem de submissao.

```pawn
native bool:mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
```

A assinatura e identica a `mysql_query`. Use `mysql_pquery` quando a ordem nao importa e voce quer maxima performance.

```pawn
// Estas 3 queries executam em paralelo
mysql_pquery(gMysql, "UPDATE stats SET kills = kills + 1 WHERE player_id = 1");
mysql_pquery(gMysql, "INSERT INTO logs (action) VALUES ('kill')");
mysql_pquery(gMysql, "SELECT * FROM rewards WHERE player_id = 1", "OnRewards");
```

## mysql_query vs mysql_pquery

| Caracteristica | mysql_query | mysql_pquery |
|---|---|---|
| Execucao | Threaded | Threaded |
| Ordem de callback | FIFO (garantida) | Sem garantia |
| Uso tipico | SELECT dependente de ordem | UPDATE, INSERT, fire-and-forget |
| Performance | Boa (reordena no dispatch) | Maxima (sem reordenacao) |

## mysql_format

Formatacao printf-like de queries com escape automatico de strings.

```pawn
native mysql_format(connId, dest[], max_len, const format[], {Float,_}:...);
```

**Retorno:** comprimento da string formatada.

### Especificadores

| Especificador | Tipo | Descricao |
|---|---|---|
| `%d` ou `%i` | int | Numero inteiro |
| `%f` | float | Numero decimal (4 casas) |
| `%s` | string | String com **escape automatico** (seguro contra SQL injection) |
| `%e` | string | Alias de `%s` (escape automatico) |
| `%r` | string | String **raw** sem escape (use apenas com valores confiaveis) |
| `%%` | — | Literal `%` |

### Exemplo

```pawn
new query[256], name[] = "O'Malley";
mysql_format(gMysql, query, sizeof(query), "SELECT * FROM players WHERE name = '%s'", name);
// Resultado: SELECT * FROM players WHERE name = 'O\'Malley'

mysql_query(gMysql, query, "OnPlayerFound");
```

### Combinando tipos

```pawn
new query[256];
mysql_format(gMysql, query, sizeof(query),
    "INSERT INTO scores (player_id, score, name) VALUES (%d, %f, '%s')",
    playerid, 99.5, "Teste"
);
```

> **Importante:** `%s` e `%e` sempre escapam a string. Se voce precisa inserir um valor sem escape (como um nome de tabela vindo de uma fonte confiavel), use `%r`. Nunca use `%r` com input do usuario.

## mysql_escape_string

Escapa uma string para uso seguro em SQL. Funcao pura, nao requer conexao.

```pawn
native bool:mysql_escape_string(const src[], dest[], max_len = sizeof(dest));
```

```pawn
new escaped[128], input[] = "It's a test\"";
mysql_escape_string(input, escaped);
// escaped = "It\'s a test\\\""
```

Caracteres escapados: `\0`, `\n`, `\r`, `\\`, `'`, `"`, `\x1a` (EOF).

## Limites

| Limite | Valor | Descricao |
|---|---|---|
| Rows por resultado | 100.000 | Resultados maiores sao truncados com warning |

## Queries pendentes

```pawn
native mysql_unprocessed_queries();
```

Retorna o numero total de queries em execucao + aguardando dispatch.

```pawn
printf("Queries pendentes: %d", mysql_unprocessed_queries());
```
