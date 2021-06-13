use hex::encode;
use log::{error, info, warn, debug};
use serde_json::{from_value, Value};
use tokio_postgres::Client;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::algo::hash::{format_block_template, get_hashing_blob_from_template};
use crate::constants::strings::JOB_STATE_FINISHED;
use crate::data::sql::{ADD_PAYMENT_SQL,
                       CREATE_JOB_SQL,
                       GET_ACCOUNTS_FOR_PAYOUT_SQL,
                       GET_JOB_FOR_MINER,
                       INSERT_BLOCK_TEMPLATE_SQL,
                       INSERT_MINER_BLOCK_TEMPLATE_SQL,
                       LOGIN_MINER_SQL,
                       UPDATE_JOB_STATE_SQL,
                       UPDATE_JOB_SUBMIT_SQL};
use crate::structs::{PayoutDTO,
                     BlockTemplate,
                     Config,
                     JobDTO,
                     LoginMessage,
                     MinerDTO,
                     MinerTemplateMessage,
                     TransferResponse};
use crate::util::generate_pool_nonce;

async fn run_query(db: &Client, statement: &str, params: &[&(dyn ToSql + Sync)]) -> bool {
    let result = db.query(statement, params).await;
    if result.is_err() {
        debug!("query failed with error: {}", result.err().unwrap());
        return false;
    }
    return true;
}

async fn get_object(db: &Client, statement: &str, params: &[&(dyn ToSql + Sync)]) -> Option<Value> {
    let result = db.query(statement, params).await;
    if result.is_err() {
        debug!("query failed with error: {}", result.err().unwrap());
        return None;
    }
    let row_vec = result.unwrap();
    if row_vec.is_empty() {
        debug!("no rows returned");
        return None;
    }
    return Some(row_vec[0].get(0));
}

async fn get_objects(db: &Client, statement: &str, params: &[&(dyn ToSql + Sync)]) -> Option<Vec<Value>> {
    let result = db.query(statement, params).await;
    if result.is_err() {
        debug!("query failed with error: {}", result.err().unwrap());
        return None;
    }
    let row_vec = result.unwrap();
    if row_vec.is_empty() {
        debug!("no rows returned");
        return None;
    }
    let val_vec = row_vec.iter().map(|v| v.get(0)).collect::<Vec<Value>>();
    return Some(val_vec);
}

pub async fn login_miner(db: &Client, config: &Config, msg: &LoginMessage) -> Option<MinerDTO> {
    let result = get_object(db, LOGIN_MINER_SQL, &[
        &msg.host,
        &msg.port,
        &msg.wallet,
        &msg.rigid,
        &msg.host,
        &msg.port,
        &(config.pool_stats_window_seconds as f64)
    ]).await;
    if result.is_none() {
        info!("unable to insert miner with host: {} port: {} wallet: {} rigid: {}",
              msg.host, msg.port, msg.wallet, msg.rigid);
        return None;
    }
    let miner = from_value(result.unwrap());
    if miner.is_err() {
        info!("unable to insert miner with host: {} port: {} wallet: {} rigid: {}",
              msg.host, msg.port, msg.wallet, msg.rigid);
        return None;
    }
    return Some(miner.unwrap());
}

pub async fn get_job_for_miner(db: &Client, client_id: &Uuid, job_id: &Uuid) -> Option<JobDTO> {
    let result = get_object(db, GET_JOB_FOR_MINER, &[
        &job_id,
        &client_id
    ]).await;
    if result.is_none() {
        warn!("unable to find job with job_id {} for miner with client_id: {}",
              job_id.to_string(), client_id.to_string());
        return None;
    }
    let job_dto_result = from_value(result.unwrap());
    if job_dto_result.is_err() {
        return None;
    }
    return job_dto_result.unwrap();
}

pub async fn create_job(db: &Client, config: &Config, miner: &MinerDTO) -> Option<JobDTO> {
    let pool_nonce = generate_pool_nonce(config);
    let mut result = get_object(db, CREATE_JOB_SQL, &[
        &(config.pool_stats_window_seconds as i32),
        &(config.pool_min_difficulty as i32),
        &miner.id,
        &(config.pool_stats_window_seconds as f64),
        &miner.id,
        &pool_nonce,
    ]).await;
    if result.is_none() {
        warn!("unable to create job for miner with client_id: {}", miner.client_id.to_string());
        return None;
    }
    let job_result = from_value(result.unwrap());
    if job_result.is_err() {
        warn!("unable to create job. got error: {}", job_result.err().unwrap());
        return None;
    }
    let mut job: JobDTO = job_result.unwrap();
    let template_response = format_block_template(config,
                                                  &job.blocktemplate_blob,
                                                  &job.pool_nonce,
                                                  &job.reserved_offset);
    if template_response.is_none() {
        error!("unable to format block template");
        return None;
    }
    let template = template_response.unwrap();
    let hashing_blob = get_hashing_blob_from_template(&template);
    let template_string = encode(template.clone());
    job.blocktemplate_blob = template_string;
    job.blockhashing_blob = hashing_blob.clone();
    return Some(job);
}

pub async fn update_miner_block_template(db: &Client, tm: &MinerTemplateMessage) -> bool {
    return run_query(db, INSERT_MINER_BLOCK_TEMPLATE_SQL, &[
        &tm.blob,
        &tm.difficulty,
        &tm.height,
        &tm.prev_hash
    ]).await;
}

pub async fn insert_block_template(db: &Client, bt: &BlockTemplate) -> Option<BlockTemplate> {
    let result = get_object(db, INSERT_BLOCK_TEMPLATE_SQL, &[
        &bt.blockhashing_blob,
        &bt.blocktemplate_blob,
        &bt.reserved_offset,
        &bt.reserved_size,
        &bt.difficulty,
        &bt.height,
        &bt.expected_reward,
        &bt.previous_hash,
        &bt.seed_hash
    ]).await;
    if result.is_none() {
        return None;
    }
    let bt = from_value(result.unwrap());
    if bt.is_err() {
        return None;
    }
    return Some(bt.unwrap());
}

pub async fn update_miner_job_state(db: &Client, job_id: &Uuid, state: &str) -> bool {
    return run_query(db, UPDATE_JOB_STATE_SQL, &[
        &state,
        &job_id
    ]).await;
}

pub async fn update_miner_job_submit_success(db: &Client, job_id: &Uuid, calculated_difficulty: &i64) -> bool {
    return run_query(db, UPDATE_JOB_SUBMIT_SQL, &[
        &JOB_STATE_FINISHED,
        &calculated_difficulty,
        &job_id
    ]).await;
}

pub async fn add_payment(db: &Client, wallet: &str, amount: &i64) -> bool {
    return run_query(db, ADD_PAYMENT_SQL, &[
        &wallet,
        &amount,
        &wallet,
    ]).await;
}

pub async fn get_accounts_for_payout(db: &Client, config: &Config) -> Option<Vec<PayoutDTO>> {
    let result = get_objects(db, GET_ACCOUNTS_FOR_PAYOUT_SQL, &[
        &(config.auto_payment_min_balance_atomic_units as i64),
        &(config.manual_payment_min_balance_atomic_units as i64),
    ]).await;
    if result.is_none() {
        return None;
    }
    let result_vec = result.unwrap();
    let mut payouts_vec: Vec<PayoutDTO> = Vec::new();
    for result_val in result_vec {
        let result_cast = from_value(result_val.clone());
        if result_cast.is_err() {
            return None;
        }
        payouts_vec.push(result_cast.unwrap());
    }
    return Some(payouts_vec);
}


