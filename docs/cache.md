# Cache

O sistema de cache armazena os resultados das queries. Quando um callback de query e chamado, o cache da query fica automaticamente disponivel para leitura.

## Ciclo de vida do cache

1. Uma query e executada na thread
2. O resultado e armazenado em um `CacheEntry`
3. No `process_tick`, o cache e empilhado (**push**)
4. O callback e chamado — dentro dele, as funcoes `cache_*` acessam o resultado
5. Apos o callback retornar, o cache e desempilhado (**pop**)

```
mysql_query → [thread] → resultado → push cache → callback() → pop cache
```

> Fora de um callback de query, as funcoes `cache_*` retornam valores de erro (-1, false, 0.0) a menos que voce use `cache_set_active` com um cache salvo.

## Leitura de resultados

### Dimensoes

```pawn
native cache_get_row_count();     // Numero de linhas (-1 se sem cache)
native cache_get_field_count();   // Numero de colunas (-1 se sem cache)
```

```pawn
forward OnQueryDone();
public OnQueryDone() {
    new rows = cache_get_row_count();
    new fields = cache_get_field_count();
    printf("Resultado: %d rows x %d fields", rows, fields);
}
```

### Valor por indice (row, col)

```pawn
native bool:cache_get_value_index(row, col, dest[], max_len = sizeof(dest));
native cache_get_value_index_int(row, col);
native Float:cache_get_value_index_float(row, col);
```

```pawn
forward OnResult();
public OnResult() {
    new rows = cache_get_row_count();
    for (new i = 0; i < rows; i++) {
        new id = cache_get_value_index_int(i, 0);
        new name[64];
        cache_get_value_index(i, 1, name);
        printf("ID: %d, Nome: %s", id, name);
    }
}
```

### Valor por nome de coluna (row, field_name)

```pawn
native bool:cache_get_value_name(row, const field_name[], dest[], max_len = sizeof(dest));
native cache_get_value_name_int(row, const field_name[]);
native Float:cache_get_value_name_float(row, const field_name[]);
```

```pawn
forward OnPlayerData();
public OnPlayerData() {
    if (cache_get_row_count() > 0) {
        new name[MAX_PLAYER_NAME];
        cache_get_value_name(0, "username", name);

        new level = cache_get_value_name_int(0, "level");
        new Float:score = cache_get_value_name_float(0, "score");

        printf("%s - Level %d - Score %.2f", name, level, score);
    }
}
```

> A busca por nome e case-insensitive: `"Username"`, `"username"` e `"USERNAME"` funcionam igualmente.

### Verificacao de NULL

```pawn
native bool:cache_is_value_index_null(row, col);
native bool:cache_is_value_name_null(row, const field_name[]);
```

```pawn
if (!cache_is_value_name_null(0, "email")) {
    new email[128];
    cache_get_value_name(0, "email", email);
    printf("Email: %s", email);
} else {
    printf("Email nao definido");
}
```

## Metadados

### Colunas

```pawn
native bool:cache_get_field_name(field_idx, dest[], max_len = sizeof(dest));
native cache_get_field_type(field_idx);   // Tipo MySQL (ver mysql_com.h)
native cache_get_field_count();
```

```pawn
new fields = cache_get_field_count();
for (new i = 0; i < fields; i++) {
    new name[64];
    cache_get_field_name(i, name);
    printf("Coluna %d: %s (tipo %d)", i, name, cache_get_field_type(i));
}
```

### Resultados de escrita (INSERT/UPDATE/DELETE)

```pawn
native cache_affected_rows();    // Linhas afetadas
native cache_insert_id();        // Last insert ID (auto_increment)
native cache_warning_count();    // Warnings do MySQL
```

```pawn
forward OnPlayerInserted();
public OnPlayerInserted() {
    new id = cache_insert_id();
    printf("Novo jogador inserido com ID: %d", id);
}

forward OnPlayersDeleted();
public OnPlayersDeleted() {
    printf("Jogadores removidos: %d", cache_affected_rows());
}
```

### Debug

```pawn
native cache_get_query_exec_time();   // Tempo de execucao em ms
native bool:cache_get_query_string(dest[], max_len = sizeof(dest));
```

```pawn
printf("Query executada em %d ms", cache_get_query_exec_time());

new query[512];
cache_get_query_string(query);
printf("Query: %s", query);
```

## Cache persistente (save/restore)

Por padrao, o cache e destruido apos o callback. Use `cache_save` para preservar o resultado.

### cache_save

```pawn
native cache_save();   // Retorna cache_id (>= 1) ou 0 se falhar
```

### cache_delete

```pawn
native bool:cache_delete(cache_id);
```

### cache_set_active / cache_unset_active

Ativa manualmente um cache salvo. Enquanto ativo, todas as funcoes `cache_*` leem deste cache.

```pawn
native bool:cache_set_active(cache_id);
native bool:cache_unset_active();
```

### Exemplo completo

```pawn
new gSavedCache;

forward OnDataLoaded();
public OnDataLoaded() {
    // Salva o cache para uso posterior
    gSavedCache = cache_save();
    printf("Cache salvo com ID: %d", gSavedCache);
}

// Em outro momento...
stock UseSavedData() {
    if (!cache_is_valid(gSavedCache)) {
        printf("Cache expirado!");
        return;
    }

    cache_set_active(gSavedCache);

    new rows = cache_get_row_count();
    printf("Dados salvos: %d rows", rows);

    // ... ler dados ...

    cache_unset_active();
}

// Quando nao precisar mais
stock CleanupCache() {
    cache_delete(gSavedCache);
    gSavedCache = 0;
}
```

### Verificacao de estado

```pawn
native bool:cache_is_any_active();    // Algum cache esta ativo?
native bool:cache_is_valid(cache_id); // O cache salvo ainda existe?
```

## Limites

| Limite | Valor |
|---|---|
| Caches salvos simultaneos | 1024 |
| Rows por resultado | 100.000 |

Se o limite de caches salvos for atingido, `cache_save` retorna `0` e um warning e logado.
Resultados com mais de 100.000 rows sao truncados automaticamente com warning.
