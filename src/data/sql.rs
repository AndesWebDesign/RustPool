///////////////////////////////////////////////////////////////////////////////////////////////////
////                                       SQL                                                 ////
///////////////////////////////////////////////////////////////////////////////////////////////////
pub const CREATE_ACCOUNT_TABLE_SQL: &str = "\
CREATE TABLE account ( \
    id                     BIGSERIAL PRIMARY KEY, \
    wallet                 TEXT UNIQUE NOT NULL, \
    balance                BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0), \
    total_paid             BIGINT NOT NULL DEFAULT 0 CHECK (total_paid >= 0), \
    wants_payout           BOOLEAN NOT NULL DEFAULT FALSE, \
    banned                 BOOLEAN NOT NULL DEFAULT FALSE, \
    created_on             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP \
);";

pub const CREATE_MINER_TABLE_SQL: &str = "\
CREATE TABLE miner ( \
    id                     BIGSERIAL PRIMARY KEY, \
    account_fk             BIGINT NOT NULL REFERENCES account(id), \
    client_id              UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(), \
    host                   TEXT NOT NULL, \
    port                   INTEGER NOT NULL CHECK (port > 0 AND port <= 65535), \
    wallet                 TEXT NOT NULL, \
    rigid                  TEXT NOT NULL, \
    banned                 BOOLEAN NOT NULL DEFAULT FALSE, \
    created_on             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, \
    UNIQUE (wallet, rigid) \
);";

pub const CREATE_BLOCK_TEMPLATE_TABLE_SQL: &str = "\
CREATE TABLE block_template ( \
    id                     BIGSERIAL PRIMARY KEY, \
    blockhashing_blob      TEXT, \
    blocktemplate_blob     TEXT NOT NULL, \
    reserved_offset        INTEGER CHECK (reserved_offset > 0), \
    reserved_size          INTEGER CHECK (reserved_size > 0), \
    difficulty             BIGINT NOT NULL CHECK (reserved_offset > 0), \
    height                 BIGINT NOT NULL CHECK (height > 0), \
    expected_reward        BIGINT CHECK (expected_reward > 0), \
    previous_hash          TEXT NOT NULL, \
    seed_hash              TEXT NOT NULL, \
    next_seed_hash         TEXT, \
    origin                 TEXT NOT NULL DEFAULT 'BACKEND', \
    created_on             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP \
);";

pub const CREATE_JOB_TABLE_SQL: &str = "\
CREATE TABLE job ( \
    id                     BIGSERIAL PRIMARY KEY, \
    job_id                 UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(), \
    miner_fk               BIGINT NOT NULL REFERENCES miner(id), \
    miner_mode             TEXT NOT NULL DEFAULT 'NORMAL', \
    target                 BIGINT NOT NULL CHECK (target > 0), \
    template_fk            BIGINT REFERENCES block_template(id), \
    pool_nonce             TEXT NOT NULL, \
    nonce                  TEXT, \
    calculated_difficulty  BIGINT CHECK (calculated_difficulty > 0), \
    state                  TEXT NOT NULL DEFAULT 'CREATED', \
    created_on             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP \
);";

pub const CREATE_PAYMENT_TABLE_SQL: &str = "\
CREATE TABLE payment ( \
    id                     BIGSERIAL PRIMARY KEY, \
    account_fk             BIGINT NOT NULL REFERENCES account(id), \
    amount                 BIGINT NOT NULL DEFAULT 0 CHECK (amount >= 0), \
    created_on             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP \
);";

pub const LOGIN_MINER_SQL: &str = "\
WITH a AS ( \
    INSERT INTO account (wallet) \
    VALUES ($1) \
    ON CONFLICT DO NOTHING \
    RETURNING \
        id,\
        banned \
), \
m AS ( \
    INSERT INTO miner (account_fk, host, port, wallet, rigid) \
    VALUES ((SELECT id FROM a), $1, $2, $3, $4) \
    ON CONFLICT (wallet, rigid) DO UPDATE \
        SET host = $5, \
            port = $6 \
    RETURNING \
        id, \
        account_fk, \
        client_id, \
        host, \
        port, \
        wallet, \
        rigid, \
        banned, \
        created_on \
) \
SELECT \
    json_build_object( \
        'id', m.id,  \
        'client_id', m.client_id, \
        'host', m.host, \
        'port', m.port, \
        'wallet', m.wallet, \
        'rigid', m.rigid, \
        'banned', (m.banned OR a.banned), \
        'created_on', m.created_on, \
        'all_jobs', count(j.job_id), \
        'open_jobs', count(j.job_id) FILTER ( \
                WHERE j.state = 'CREATED' \
                ), \
        'error_jobs', count(j.job_id) FILTER ( \
                WHERE j.state = 'ERROR' \
                )
    ) AS miner \
FROM m \
LEFT JOIN a \
    ON a.id = m.account_fk \
LEFT JOIN job j \
    ON m.id = j.miner_fk \
    AND j.created_on > CURRENT_TIMESTAMP - make_interval(0,0,0,0,0,0,$7) \
GROUP BY \
    m.id, \
    m.client_id, \
    m.host, \
    m.port, \
    m.wallet, \
    m.rigid, \
    m.banned, \
    a.banned, \
    m.created_on;";

pub const CREATE_JOB_SQL: &str = "\
WITH lt AS (SELECT * \
            FROM block_template \
            WHERE origin = 'BACKEND' \
            ORDER BY created_on DESC \
            LIMIT 1), \
     hr AS (SELECT GREATEST(SUM(COALESCE(target, 0)) / $1::INTEGER, $2::INTEGER) AS diff, \
            (SELECT id FROM lt) AS bt_id \
            FROM job \
            WHERE miner_fk = $3 \
              AND created_on > CURRENT_TIMESTAMP - make_interval(0,0,0,0,0,0,$4) \
            LIMIT 1), \
     cd AS (SELECT LEAST(difficulty, diff) AS difficulty FROM lt LEFT JOIN hr ON id = bt_id) \
INSERT INTO job (miner_fk, template_fk, pool_nonce, target) \
VALUES ($5, (SELECT id FROM lt), $6, (SELECT difficulty FROM cd)) \
ON CONFLICT DO NOTHING \
RETURNING \
json_build_object( \
    'job_id', job_id, \
    'state', state, \
    'pool_nonce', pool_nonce, \
    'target', target, \
    'blockhashing_blob', (SELECT blockhashing_blob FROM lt), \
    'blocktemplate_blob', (SELECT blocktemplate_blob FROM lt), \
    'height', (SELECT height FROM lt), \
    'previous_hash', (SELECT previous_hash FROM lt), \
    'seed_hash', (SELECT seed_hash FROM lt), \
    'reserved_offset', (SELECT reserved_offset FROM lt), \
    'difficulty', (SELECT difficulty FROM lt)) AS job;";

pub const TEST_SCHEMA_EXISTS_SQL: &str = "\
SELECT 1 \
FROM information_schema.tables \
WHERE table_schema = 'public' \
    AND table_name = 'miner' \
LIMIT 1;";

pub const INSERT_BLOCK_TEMPLATE_SQL: &str = "\
INSERT INTO block_template (blockhashing_blob, \
                            blocktemplate_blob, \
                            reserved_offset, \
                            reserved_size, \
                            difficulty, \
                            height, \
                            expected_reward, \
                            previous_hash,\
                            seed_hash) \
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
ON CONFLICT DO NOTHING \
RETURNING \
    json_build_object( \
        'blockhashing_blob', blockhashing_blob, \
        'blocktemplate_blob', blocktemplate_blob, \
        'reserved_offset', reserved_offset, \
        'reserved_size', reserved_size, \
        'difficulty', difficulty, \
        'height', height, \
        'expected_reward', expected_reward, \
        'origin', origin, \
        'previous_hash', previous_hash, \
        'seed_hash', seed_hash, \
        'next_seed_hash', next_seed_hash \
) AS bt;";

pub const INSERT_MINER_BLOCK_TEMPLATE_SQL: &str = "\
WITH ir AS (INSERT INTO block_template (blocktemplate_blob, \
                                        difficulty, \
                                        height, \
                                        previous_hash, \
                                        origin) \
            VALUES ($1, $2, $3, $4, 'MINER') \
            ON CONFLICT DO NOTHING \
            RETURNING id) \
UPDATE job SET template_fk = ir.id \
WHERE miner_fk = $1 \
    AND job_id = $2 \
RETURNING \
    json_build_object( \
        'blockhashing_blob', blockhashing_blob, \
        'blocktemplate_blob', blocktemplate_blob, \
        'reserved_offset', reserved_offset, \
        'reserved_size', reserved_size, \
        'difficulty', difficulty, \
        'height', height, \
        'expected_reward', expected_reward, \
        'origin', origin, \
        'previous_hash', previous_hash, \
        'seed_hash', seed_hash, \
        'next_seed_hash', next_seed_hash \
) AS bt;";

pub const GET_JOB_FOR_MINER: &str = "\
SELECT \
    json_build_object( \
        'job_id', job_id, \
        'state', state, \
        'pool_nonce', pool_nonce, \
        'target', target, \
        'blockhashing_blob', (SELECT blockhashing_blob FROM bt), \
        'blocktemplate_blob', (SELECT blocktemplate_blob FROM bt), \
        'height', (SELECT height FROM bt), \
        'previous_hash', (SELECT previous_hash FROM bt), \
        'seed_hash', (SELECT seed_hash FROM bt), \
        'reserved_offset', (SELECT reserved_offset FROM bt), \
        'difficulty', (SELECT difficulty FROM bt) \
    ) AS job \
FROM job j \
LEFT JOIN block_template bt \
    ON bt.id = j.template_fk \
LEFT JOIN miner m \
    ON m.id = j.miner_fk \
WHERE j.job_id = $1 \
    AND m.client_id = $2 \
    AND j.state = 'CREATED';";

pub const UPDATE_JOB_STATE_SQL: &str = "\
UPDATE job \
SET state = $1 \
WHERE job_id = $2;";

pub const UPDATE_JOB_SUBMIT_SQL: &str = "\
UPDATE job \
SET state = $1 \
    AND calculated_difficulty = $2 \
WHERE job_id = $3;";

pub const ADD_PAYMENT_SQL: &str = "\
WITH new_payment AS ( \
    INSERT INTO payment (account_fk, amount) \
    VALUES ($1, $2) \
    RETURNING amount \
) \
UPDATE account \
SET balance = 0 \
    AND total_paid = total_paid + new_payment.amount
WHERE wallet = $3;";

pub const GET_ACCOUNTS_FOR_PAYOUT_SQL: &str = "\
SELECT \
    json_build_object( \
        'id', a.id, \
        'balance', a.balance \
    ) AS account \
FROM account a \
WHERE a.balance > $1 \
    OR (a.wants_payout AND a.balance > $2);";
