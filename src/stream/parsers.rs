use std::net::SocketAddr;
use std::str::FromStr;

use serde_json::Value;
use uuid::Uuid;

use crate::config::magic::MISSING_STRING;
use crate::structs::{LoginMessage,
                     MinerTemplateMessage,
                     SubmitBlockMessage};
use crate::util::{get_i64_val_or_missing,
                  get_object_val_or_missing,
                  get_string_val_or_missing,
                  get_uuid_val_or_missing};

pub fn parse_submit_block_message(message: &Value, addr: &SocketAddr) -> Option<SubmitBlockMessage> {
    let mut parsed: SubmitBlockMessage = Default::default();
    parsed.host = addr.ip().to_string();
    parsed.port = addr.port() as i32;
    parsed.message_id = get_message_id(message);
    let params: Value = get_params(message);
    parsed.client_id = get_client_id(&params);
    parsed.job_id = get_job_id(&params);
    parsed.nonce = get_nonce(&params);
    parsed.result = get_result(&params);
    if !parsed.is_valid() {
        return None;
    }
    return Some(parsed);
}

pub async fn parse_miner_login_request(message: &Value, addr: &SocketAddr) -> Option<LoginMessage> {
    let mut parsed: LoginMessage = Default::default();
    parsed.host = addr.ip().to_string();
    parsed.port = addr.port() as i32;
    parsed.message_id = get_message_id(message);
    let params: Value = get_params(message);
    parsed.mode = get_mode(&params);
    parsed.wallet = get_wallet(&params);
    parsed.rigid = get_rigid(&params);
    if !parsed.is_valid() {
        return None;
    }
    return Some(parsed);
}

pub fn parse_miner_block_template_request(message: &Value, addr: &SocketAddr) -> Option<MinerTemplateMessage> {
    let mut parsed: MinerTemplateMessage = Default::default();
    parsed.host = addr.ip().to_string();
    parsed.port = addr.port() as i32;
    parsed.message_id = get_message_id(message);
    let params: Value = get_params(message);
    parsed.client_id = get_client_id(&params);
    parsed.job_id = get_job_id(&params);
    parsed.blob = get_blob(&params);
    parsed.height = get_height(&params);
    parsed.difficulty = get_difficulty(&params);
    parsed.prev_hash = get_prev_hash(&params);
    if !parsed.is_valid() {
        return None;
    }
    return Some(parsed);
}


pub fn get_params(message: &Value) -> Value {
    return get_object_val_or_missing(message, "params");
}

pub fn get_message_id(message: &Value) -> String {
    return get_i64_val_or_missing(message, "id").as_i64().unwrap_or(i64::MIN).to_string();
}

pub fn get_method(message: &Value) -> String {
    return get_string_val_or_missing(message, "method").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_mode(params: &Value) -> String {
    return get_string_val_or_missing(params, "mode").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_wallet(params: &Value) -> String {
    return get_string_val_or_missing(params, "login").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_rigid(params: &Value) -> String {
    return get_string_val_or_missing(params, "rigid").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_nonce(params: &Value) -> String {
    return get_string_val_or_missing(params, "nonce").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_result(params: &Value) -> String {
    return get_string_val_or_missing(params, "result").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_blob(params: &Value) -> String {
    return get_string_val_or_missing(params, "blob").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_height(params: &Value) -> i64 {
    return get_i64_val_or_missing(params, "height").as_i64().unwrap_or(i64::MIN);
}

pub fn get_difficulty(params: &Value) -> i64 {
    return get_i64_val_or_missing(params, "difficulty").as_i64().unwrap_or(i64::MIN);
}

pub fn get_prev_hash(params: &Value) -> String {
    return get_string_val_or_missing(params, "prev_hash").as_str().unwrap_or(MISSING_STRING).to_string();
}

pub fn get_client_id(params: &Value) -> Uuid {
    let uuid_str = get_uuid_val_or_missing(params, "id").as_str().unwrap_or(MISSING_STRING).to_string().clone();
    return Uuid::from_str(uuid_str.as_str()).unwrap_or(Uuid::nil());
}

pub fn get_job_id(params: &Value) -> Uuid {
    let uuid_str = get_uuid_val_or_missing(params, "job_id").as_str().unwrap_or(MISSING_STRING).to_string().clone();
    return Uuid::from_str(uuid_str.as_str()).unwrap_or(Uuid::nil());
}
