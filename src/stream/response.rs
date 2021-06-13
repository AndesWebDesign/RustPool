use hex::encode;
use log::error;
use serde_json::{json, Value};

use crate::algo::hash::format_block_template;
use crate::config::magic::{JSON_RPC_VERSION, MISSING_STRING};
use crate::structs::{Config, JobDTO, LoginMessage, MinerDTO};

pub async fn process_job_response(config: &Config,
                                  job: &JobDTO,
                                  miner: &MinerDTO,
                                  msg: &LoginMessage,
                                  result: bool) -> Value {
    let response: Value;
    match msg.mode.as_str() {
        MISSING_STRING => {
            // mode field is missing => normal mode
            let bt_response = format_block_template(config,
                                                    &job.blocktemplate_blob,
                                                    &job.pool_nonce,
                                                    &job.reserved_offset);
            if bt_response.is_none() {
                return get_error_body("could not get job", msg.message_id.as_str());
            }
            if result {
                response = json!({
                    "result": {
                        "job": {
                            "job_id": job.job_id,
                            "target": job.target,
                            "height": job.height,
                            "blob": encode(bt_response.unwrap())
                        },
                        "id": miner.client_id.to_string(),
                        "status": "OK"
                    },
                    "jsonrpc": JSON_RPC_VERSION,
                    "error": null,
                    "id": msg.message_id
                });
            } else {
                response = json!({
                    "params": {
                        "job_id": job.job_id,
                        "target": job.target,
                        "height": job.height,
                        "blob": encode(bt_response.unwrap()),
                        "id": miner.client_id.to_string()
                    },
                    "jsonrpc": JSON_RPC_VERSION,
                    "method": "job"
                });
            }
        }
        "self-select" => {
            // self select mode => just return pool_nonce
            if result {
                response = json!({
                    "result": {
                        "job": {
                            "job_id": job.job_id,
                            "target": job.target,
                            "pool_wallet": config.wallet,
                            "extra_nonce": job.pool_nonce
                        },
                        "id": miner.client_id.to_string(),
                        "status": "OK"
                    },
                    "jsonrpc": JSON_RPC_VERSION,
                    "error": null,
                    "id": msg.message_id
                });
            } else {
                response = json!({
                    "params": {
                        "job_id": job.job_id,
                        "target": job.target,
                        "pool_wallet": config.wallet,
                        "extra_nonce": job.pool_nonce,
                        "id": miner.client_id.to_string()
                    },
                    "jsonrpc": JSON_RPC_VERSION,
                    "method": "job"
                });
            }
        }
        _ => {
            error!("mode not supported: {}", msg.mode.as_str());
            return json!(MISSING_STRING);
        }
    }
    return response;
}

pub fn get_status_body(status: &str, message_id: &str) -> Value {
    return json!({
        "id": message_id,
        "jsonrpc": JSON_RPC_VERSION,
        "error": null,
        "status": status
    });
}

pub fn get_error_body(error_message: &str, message_id: &str) -> Value {
    return json!({
        "id": message_id,
        "jsonrpc": JSON_RPC_VERSION,
        "error": {
            "code": -1,
            "message": error_message
        }
    });
}
