use std::net::SocketAddr;

use hex::decode;
use log::{error, warn};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use serde_json::Value;
use tokio_postgres::Client as Db;

use crate::algo::hash::{add_miner_nonce, calculate_difficulty, format_block_template};
use crate::algo::randomx::get_rx_hash;
use crate::config::magic::MISSING_STRING;
use crate::constants::strings::{JOB_STATE_CREATED,
                                JOB_STATE_ERROR,
                                MESSAGE_METHOD_BLOCK_TEMPLATE,
                                MESSAGE_METHOD_KEEPALIVE,
                                MESSAGE_METHOD_LOGIN,
                                MESSAGE_METHOD_SUBMIT,
                                STATUS_KEEPALIVE,
                                STATUS_OK};
use crate::data::api::{create_job,
                       get_job_for_miner,
                       login_miner,
                       update_miner_block_template,
                       update_miner_job_state,
                       update_miner_job_submit_success};
use crate::stream::http::{init_worker_listener,
                          read_message,
                          write_message};
use crate::stream::parsers::{get_message_id,
                             get_method,
                             parse_miner_block_template_request,
                             parse_miner_login_request,
                             parse_submit_block_message};
use crate::stream::response::{get_error_body,
                              get_status_body,
                              process_job_response};
use crate::stream::rpc::submit_block;
use crate::structs::Config;

async fn handle_login_request(message: &Value,
                              config: &Config,
                              db: &Db,
                              addr: &SocketAddr) -> Value {
    let parse_result = parse_miner_login_request(message, addr).await;
    if parse_result.is_none() {
        error!("{}", "could not parse login request");
        return get_error_body("invalid request", &*get_message_id(message));
    }
    let msg = parse_result.unwrap();
    let miner_response = login_miner(db, config, &msg).await;
    if miner_response.is_none() {
        error!("{}", "miner not found");
        return get_error_body("miner not found", &msg.message_id);
    }
    let miner = miner_response.unwrap();
    if !miner.can_have_job(&config) {
        warn!("{}", "miner cannot have job");
        return get_error_body("job not available", &msg.message_id);
    }
    let job_result = create_job(db, config, &miner).await;
    if job_result.is_none() {
        error!("unable to create job");
        return get_error_body("job not available", &msg.message_id);
    }
    let job = job_result.unwrap();
    return process_job_response(config, &job, &miner, &msg, true).await;
}

async fn handle_block_template_request(message: &Value,
                                       config: &Config,
                                       db: &Db,
                                       addr: &SocketAddr) -> Value {
    let parse_result = parse_miner_block_template_request(message, addr);
    if parse_result.is_none() {
        error!("unable to parse block template message");
        return get_error_body("invalid message", &*get_message_id(message));
    }
    let msg = parse_result.unwrap();
    if !config.allow_self_select {
        return get_error_body("template self select not allowed", &msg.message_id);
    }
    if msg.blob.is_empty() {
        error!("could not parse block template");
        return get_error_body("could not parse block template", &msg.message_id);
    }
    let job_result = get_job_for_miner(db, &msg.client_id, &msg.job_id).await;
    if job_result.is_none() {
        warn!("unable to find job for miner with client id: {}", msg.client_id.to_string());
        return get_error_body("job not available", &msg.message_id);
    }
    let job = job_result.unwrap();
    if job.state.ne(JOB_STATE_CREATED) {
        warn!("job not open, maybe a repeat request");
        return get_error_body("no miner exists with id: {}", &msg.message_id);
    }
    if !update_miner_block_template(db, &msg).await {
        error!("could not update miner block template");
        if !update_miner_job_state(db, &job.job_id, JOB_STATE_ERROR).await {
            return get_error_body("block template not accepted", &msg.message_id);
        }
        return get_error_body("block template not accepted", &msg.message_id);
    }
    return get_status_body(STATUS_OK, &msg.message_id);
}

async fn handle_submit_block_request(message: &Value,
                                     config: &Config,
                                     db: &Db,
                                     addr: &SocketAddr) -> Value {
    let parse_result = parse_submit_block_message(message, addr);
    if parse_result.is_none() {
        error!("could not parse submit block request");
        return get_error_body("could not parse", &*get_message_id(message));
    }
    let msg = parse_result.unwrap();
    let job_result = get_job_for_miner(db, &msg.client_id, &msg.job_id).await;
    if job_result.is_none() {
        warn!("no job found for miner with client_id: {}", msg.client_id);
        return get_error_body("no job found", &msg.message_id);
    }
    let job = job_result.unwrap();
    let mut bt_response = format_block_template(config,
                                                &job.blocktemplate_blob,
                                                &job.pool_nonce,
                                                &job.reserved_offset);
    if bt_response.is_none() {
        return get_error_body("block invalid", &msg.message_id);
    }
    bt_response = add_miner_nonce(&bt_response.unwrap(), &msg.nonce);
    if bt_response.is_none() {
        return get_error_body("block invalid", &msg.message_id);
    }
    let hash = get_rx_hash(config,
                           bt_response.unwrap().as_slice(),
                           decode(job.seed_hash).unwrap().as_slice());
    if !hash.eq(&msg.result) {
        error!("submitted hash does not match computed hash");
        if !update_miner_job_state(db, &job.job_id, JOB_STATE_ERROR).await {
            return get_error_body("block invalid", &msg.message_id);
        }
        return get_error_body("block invalid", &msg.message_id);
    }
    let hash_difficulty = calculate_difficulty(&hash);
    if BigInt::from(job.difficulty as u64) <= hash_difficulty {
        submit_block(config, hash.as_str()).await;
        if !update_miner_job_submit_success(db, &job.job_id, &hash_difficulty.to_i64().unwrap()).await {
            return get_error_body("could not process share", &msg.message_id);
        }
        return get_status_body(STATUS_OK, &msg.message_id);
    }
    if BigInt::from(job.target as u64) <= hash_difficulty {
        if !update_miner_job_submit_success(db, &job.job_id, &hash_difficulty.to_i64().unwrap()).await {
            return get_error_body("could not process share", &msg.message_id);
        }
        return get_status_body(STATUS_OK, &msg.message_id);
    }
    if !update_miner_job_state(db, &job.job_id, JOB_STATE_ERROR).await {
        return get_error_body("difficulty too low", &msg.message_id);
    }
    return get_error_body("difficulty too low", &msg.message_id);
}

fn handle_keepalive_request(message: &Value) -> Value {
    let message_id = get_message_id(message).to_string().clone();
    return get_status_body(STATUS_KEEPALIVE, &message_id.as_str());
}

async fn process_worker_message(message: &Value,
                                config: &Config,
                                db: &Db,
                                addr: &SocketAddr) -> Value {
    let method = get_method(message);
    if method.as_str().eq(MISSING_STRING) {
        error!("method field missing from address: {}", addr.to_string());
        return get_error_body("method field missing", &get_message_id(message));
    };
    match method.as_str() {
        MESSAGE_METHOD_LOGIN => {
            return handle_login_request(message, config, db, addr).await;
        }
        MESSAGE_METHOD_BLOCK_TEMPLATE => {
            return handle_block_template_request(message, config, db, addr).await;
        }
        MESSAGE_METHOD_SUBMIT => {
            return handle_submit_block_request(message, config, db, addr).await;
        }
        MESSAGE_METHOD_KEEPALIVE => {
            return handle_keepalive_request(message);
        }
        _ => {
            error!("method not recognized: {} from address: {}", method, addr.to_string());
            return get_error_body("method not recognized", &get_message_id(message));
        }
    }
}

pub async fn init_worker_pool_listener(config: &Config, db: &Db) {
    let listener = init_worker_listener(config).await;
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let message = &read_message(&stream).await;
        let response = process_worker_message(message, config, db, &addr).await;
        if !write_message(&stream, response).await {
            error!("unable to write message response")
        }
    }
}

