# Instalacao e configuracao

## Requisitos

- Servidor SA:MP 0.3.7+ ou [open.mp](https://open.mp)
- MySQL/MariaDB 5.7+ (recomendado 8.0+)
- Nenhuma biblioteca externa necessaria — o plugin e auto-contido

## Download

Baixe a versao mais recente em [Releases](https://github.com/NullSablex/mysql_samp/releases/latest):

| Plataforma | Arquivo |
|---|---|
| Linux (32-bit) | `mysql_samp.so` |
| Windows (32-bit) | `mysql_samp.dll` |

## Instalacao

1. Copie o binario para o diretorio `plugins/` do seu servidor
2. Copie `mysql_samp.inc` para a pasta de includes do seu compilador:
   - **Windows:** `pawno/include/` ou `qawno/include/`
   - **Linux:** `include/` (na raiz do servidor)
3. Edite o `server.cfg` (ou `config.json` no open.mp):

**Linux:**
```
plugins mysql_samp.so
```

**Windows:**
```
plugins mysql_samp.dll
```

> O plugin nao depende de `libmysqlclient`, `libssl` ou qualquer outra biblioteca do sistema. Tudo e compilado estaticamente no binario.

## Estrutura de arquivos

**Linux:**
```
servidor/
├── gamemodes/
│   └── seu_gamemode.amx
├── include/
│   └── mysql_samp.inc
├── plugins/
│   └── mysql_samp.so
├── logs/
│   └── mysql.log          ← criado automaticamente pelo plugin
└── server.cfg
```

**Windows:**
```
servidor/
├── gamemodes/
│   └── seu_gamemode.amx
├── pawno/
│   └── include/
│       └── mysql_samp.inc
├── plugins/
│   └── mysql_samp.dll
├── logs/
│   └── mysql.log          ← criado automaticamente pelo plugin
└── server.cfg
```

## Verificando a instalacao

Ao iniciar o servidor, o banner do plugin aparece no console:

```
  | mysql_samp 0.2.0 | 2026
  |-------------------------------
  | Author and maintainer: NullSablex
  |
  | Compiled: Feb 23 2026 at 14:30:00
  |-------------------------------
  | Repository: https://github.com/NullSablex/mysql_samp
```

Se o banner nao aparecer, verifique:
- O arquivo esta na pasta `plugins/` correta
- A plataforma esta correta (32-bit)
- O `server.cfg` tem o nome exato do arquivo

## Logs

O plugin cria automaticamente o diretorio `logs/` e o arquivo `logs/mysql.log`. Este arquivo contem detalhes de erros com timestamp, uteis para debug.

O console do servidor exibe apenas mensagens genericas (sem dados sensiveis como queries ou senhas).

### Niveis de log

Voce pode configurar o nivel de log em runtime:

```pawn
mysql_log(MYSQL_LOG_NONE);      // 0 - desativa todos os logs
mysql_log(MYSQL_LOG_ERROR);     // 1 - apenas erros
mysql_log(MYSQL_LOG_WARNING);   // 2 - erros + warnings
mysql_log(MYSQL_LOG_INFO);      // 3 - erros + warnings + info
mysql_log(MYSQL_LOG_ALL);       // 4 - tudo (padrao)
```
