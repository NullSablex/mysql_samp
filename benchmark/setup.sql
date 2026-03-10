-- ============================================================
-- Benchmark: mysql_samp vs R41-4
-- Setup do banco de dados
--
-- Execute este arquivo UMA VEZ antes de rodar qualquer GM:
--   mysql -u root -p nome_do_banco < setup.sql
-- ============================================================

-- Tabela usada nos SELECTs
CREATE TABLE IF NOT EXISTS `bench_test` (
    `id`    INT         NOT NULL AUTO_INCREMENT,
    `name`  VARCHAR(64) NOT NULL DEFAULT '',
    `score` INT         NOT NULL DEFAULT 0,
    PRIMARY KEY (`id`),
    INDEX `idx_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tabela usada nos INSERTs (separada para nao poluir a principal)
CREATE TABLE IF NOT EXISTS `bench_insert` (
    `id`         INT         NOT NULL AUTO_INCREMENT,
    `name`       VARCHAR(64) NOT NULL DEFAULT '',
    `score`      INT         NOT NULL DEFAULT 0,
    `created_at` TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Limpa dados de runs anteriores
TRUNCATE TABLE `bench_test`;
TRUNCATE TABLE `bench_insert`;

-- Dados de teste (100 jogadores ficticios para os SELECTs)
INSERT INTO `bench_test` (`name`, `score`) VALUES
('Alpha',     1500), ('Bravo',     2300), ('Charlie',   800),  ('Delta',     3100),
('Echo',      450),  ('Foxtrot',   1750), ('Golf',      920),  ('Hotel',     2800),
('India',     660),  ('Juliett',   1200), ('Kilo',      3400), ('Lima',      550),
('Mike',      2100), ('November',  390),  ('Oscar',     1800), ('Papa',      2950),
('Quebec',    710),  ('Romeo',     1350), ('Sierra',    4200), ('Tango',     880),
('Uniform',   2600), ('Victor',    1050), ('Whiskey',   3700), ('Xray',      490),
('Yankee',    1900), ('Zulu',      2400), ('Ace',       320),  ('Blade',     1650),
('Cruz',      2750), ('Duke',      960),  ('Edge',      1450), ('Fury',      3300),
('Grim',      780),  ('Hawk',      2200), ('Iron',      1100), ('Jack',      3900),
('King',      640),  ('Lion',      1550), ('Mars',      2850), ('Nova',      410),
('Orion',     1700), ('Pulse',     3050), ('Quad',      870),  ('Rex',       2450),
('Steel',     1250), ('Thor',      4100), ('Ultra',     590),  ('Viper',     1800),
('Wolf',      2650), ('Xenon',     1000), ('Yoshi',     3200), ('Zeus',      730),
('Apex',      1950), ('Bear',      2550), ('Cobra',     1150), ('Dart',      3600),
('Elder',     680),  ('Flash',     2100), ('Ghost',     1400), ('Havoc',     3800),
('Igor',      510),  ('Joker',     1700), ('Kraken',    2900), ('Lycan',     850),
('Mango',     1300), ('Ninja',     3500), ('Omega',     760),  ('Panther',   2250),
('Quartz',    1600), ('Ranger',    3150), ('Shadow',    920),  ('Titan',     2700),
('Umbra',     1050), ('Valkyrie',  3850), ('Wraith',    1450), ('Xeon',      2400),
('Yeti',      680),  ('Zenith',    1900), ('Archer',    2800), ('Blaze',     1100),
('Cipher',    3400), ('Dagger',    560),  ('Ember',     2050), ('Falcon',    3700),
('Golem',     890),  ('Hydra',     2300), ('Ibis',      1500), ('Jolt',      3000),
('Karma',     720),  ('Lance',     1800), ('Mirage',    2600), ('Nexus',     1250),
('Onyx',      3300), ('Phantom',   950),  ('Quill',     1700), ('Raven',     2900),
('Storm',     1350), ('Talon',     3600), ('Umbral',    770),  ('Vault',     2150);

-- Confirma o resultado
SELECT 'Setup concluido.' AS status;
SELECT COUNT(*) AS total_registros FROM `bench_test`;
