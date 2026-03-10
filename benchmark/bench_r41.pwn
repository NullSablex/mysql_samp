// ============================================================
// Benchmark: MySQL R41-4 (pBlueG/SA-MP-MySQL)
// Referencia: https://github.com/pBlueG/SA-MP-MySQL
//
// Como usar:
//   1. Execute benchmark/setup.sql no seu banco de dados
//   2. Preencha as credenciais na secao CONFIG abaixo
//   3. Compile com o compilador Pawn do SA-MP ou open.mp
//      com a include do R41-4 (a_mysql.inc)
//   4. Coloque na pasta gamemodes/ e inicie o servidor
//      com o plugin MySQL R41-4 carregado (server.cfg/config.json)
//   5. Observe o output no console
//   6. Compare com os resultados de bench_mysql_samp.pwn
//
// Dependencias:
//   - a_mysql.inc (include do R41-4)
//   - mysql.so / mysql.dll (plugin R41-4)
// ============================================================

#include <a_samp>
#include <a_mysql>

// ---- CONFIG ------------------------------------------------
#define DB_HOST     "127.0.0.1"
#define DB_USER     "root"
#define DB_PASS     "senha"
#define DB_NAME     "benchmark"
// ------------------------------------------------------------

// Rounds de cada etapa (identicos ao bench_mysql_samp.pwn)
#define ROUNDS_SELECT   500
#define ROUNDS_PSELECT  500
#define ROUNDS_INSERT   200
#define ROUNDS_FORMAT   50000

new MySQL:gMysql;

// Contadores e timers por etapa
new gDone;
new gStart;

// ============================================================
// Inicializacao
// ============================================================

main() {}

public OnGameModeInit()
{
    // R41-4 usa mysql_connect_db ou mysql_connect dependendo da versao
    // Para R41-4 a assinatura e: mysql_connect(host, user, pass, db)
    gMysql = mysql_connect(DB_HOST, DB_USER, DB_PASS, DB_NAME);
    if (!gMysql)
    {
        print("[Bench] ERRO: falha ao conectar ao MySQL.");
        print("[Bench] Verifique as credenciais em bench_r41.pwn");
        return 1;
    }

    print("");
    print("[Bench] ================================================");
    print("[Bench]  R41-4 benchmark");
    print("[Bench]  Execute setup.sql antes de iniciar");
    print("[Bench] ================================================");

    // Aguarda 500ms para a conexao estabilizar antes de comecar
    SetTimer("BenchStart", 500, false);
    return 1;
}

public OnGameModeExit()
{
    mysql_close(gMysql);
    return 1;
}

// ============================================================
// Etapa 0: inicio
// ============================================================

forward BenchStart();
public BenchStart()
{
    // Verifica se a conexao foi estabelecida de fato
    // R41-4 conecta de forma assincrona; mysql_connect retorna o handle
    // antes de confirmar a conexao. mysql_errno == 0 significa OK.
    if (mysql_errno(gMysql) != 0)
    {
        new errMsg[256];
        mysql_error(errMsg, sizeof(errMsg), gMysql);
        printf("[Bench] ERRO: conexao falhou — %s", errMsg);
        print ("[Bench] Verifique host, usuario, senha e nome do banco.");
        return;
    }

    print("[Bench] Conexao OK. Iniciando etapas...");
    BenchSelectFIFO();
}

// ============================================================
// Etapa 1: SELECT sequencial (mysql_tquery — FIFO)
// Equivalente a mysql_query do mysql_samp.
// ============================================================

stock BenchSelectFIFO()
{
    gDone  = 0;
    gStart = GetTickCount();

    printf("[Bench] Etapa 1/%d — SELECT FIFO (mysql_tquery) x%d", 4, ROUNDS_SELECT);

    for (new i = 1; i <= ROUNDS_SELECT; i++)
    {
        mysql_tquery(gMysql,
            "SELECT id, name, score FROM bench_test WHERE id = 1",
            "OnBenchSelectFIFO", "i", i);
    }
}

forward OnBenchSelectFIFO(round);
public OnBenchSelectFIFO(round)
{
    gDone++;
    if (gDone < ROUNDS_SELECT) return;

    new ms = GetTickCount() - gStart;
    print ("[Bench] --- Resultado Etapa 1 ---");
    printf("[Bench]   Queries:    %d", ROUNDS_SELECT);
    printf("[Bench]   Total:      %d ms", ms);
    printf("[Bench]   Media:      %.3f ms/query", float(ms) / float(ROUNDS_SELECT));
    printf("[Bench]   Throughput: %.1f queries/s",
        float(ROUNDS_SELECT) * 1000.0 / float(ms));
    print ("");

    BenchSelectParallel();
}

// ============================================================
// Etapa 2: SELECT paralelo (mysql_pquery)
// ============================================================

stock BenchSelectParallel()
{
    gDone  = 0;
    gStart = GetTickCount();

    printf("[Bench] Etapa 2/%d — SELECT paralelo (mysql_pquery) x%d", 4, ROUNDS_PSELECT);

    for (new i = 1; i <= ROUNDS_PSELECT; i++)
    {
        mysql_pquery(gMysql,
            "SELECT id, name, score FROM bench_test WHERE id = 1",
            "OnBenchSelectParallel", "i", i);
    }
}

forward OnBenchSelectParallel(round);
public OnBenchSelectParallel(round)
{
    gDone++;
    if (gDone < ROUNDS_PSELECT) return;

    new ms = GetTickCount() - gStart;
    print ("[Bench] --- Resultado Etapa 2 ---");
    printf("[Bench]   Queries:    %d", ROUNDS_PSELECT);
    printf("[Bench]   Total:      %d ms", ms);
    printf("[Bench]   Media:      %.3f ms/query", float(ms) / float(ROUNDS_PSELECT));
    printf("[Bench]   Throughput: %.1f queries/s",
        float(ROUNDS_PSELECT) * 1000.0 / float(ms));
    print ("");

    BenchInsert();
}

// ============================================================
// Etapa 3: INSERT paralelo (mysql_pquery)
// ============================================================

stock BenchInsert()
{
    gDone  = 0;
    gStart = GetTickCount();

    printf("[Bench] Etapa 3/%d — INSERT paralelo (mysql_pquery) x%d", 4, ROUNDS_INSERT);

    new query[192];
    for (new i = 0; i < ROUNDS_INSERT; i++)
    {
        mysql_format(gMysql, query, sizeof(query),
            "INSERT INTO bench_insert (name, score) VALUES ('bench_%d', %d)",
            i, i * 7);
        mysql_pquery(gMysql, query, "OnBenchInsert", "i", i);
    }
}

forward OnBenchInsert(round);
public OnBenchInsert(round)
{
    gDone++;
    if (gDone < ROUNDS_INSERT) return;

    new ms = GetTickCount() - gStart;
    print ("[Bench] --- Resultado Etapa 3 ---");
    printf("[Bench]   Queries:    %d", ROUNDS_INSERT);
    printf("[Bench]   Total:      %d ms", ms);
    printf("[Bench]   Media:      %.3f ms/query", float(ms) / float(ROUNDS_INSERT));
    printf("[Bench]   Throughput: %.1f queries/s",
        float(ROUNDS_INSERT) * 1000.0 / float(ms));
    print ("");

    BenchFormat();
}

// ============================================================
// Etapa 4: mysql_format com escape
// No R41-4, %s NAO escapa — usamos %e para escapar,
// que e o equivalente ao %s do mysql_samp.
// ============================================================

stock BenchFormat()
{
    printf("[Bench] Etapa 4/%d — mysql_format com escape x%d (sincrono)", 4, ROUNDS_FORMAT);

    new query[256];
    new dangerous[] = "'; DROP TABLE bench_test; -- O'Brien & Co.";
    new start = GetTickCount();

    for (new i = 0; i < ROUNDS_FORMAT; i++)
    {
        // Nota: no R41-4 '%s' e RAW (sem escape). Para escapar use '%e'.
        // Usamos %e aqui para garantir comparacao justa com mysql_samp (%s).
        mysql_format(gMysql, query, sizeof(query),
            "SELECT * FROM bench_test WHERE name = '%e' AND score > %d",
            dangerous, i);
    }

    new ms = GetTickCount() - start;
    print ("[Bench] --- Resultado Etapa 4 ---");
    printf("[Bench]   Iteracoes:  %d", ROUNDS_FORMAT);
    printf("[Bench]   Total:      %d ms", ms);
    printf("[Bench]   Media:      %.4f ms/chamada", float(ms) / float(ROUNDS_FORMAT));
    printf("[Bench]   Throughput: %.0f format/s",
        float(ROUNDS_FORMAT) * 1000.0 / float(ms));
    print ("");
    print ("[Bench] ================================================");
    print ("[Bench]  Benchmark concluido.");
    print ("[Bench]  Compare com bench_mysql_samp.pwn para ver a diferenca.");
    print ("[Bench] ================================================");
}
