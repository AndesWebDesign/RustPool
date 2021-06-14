use std::str::{from_utf8, FromStr};

use digest_auth::{AuthContext, AuthorizationHeader, Error, WwwAuthenticateHeader};
use log::{debug, info, warn};
use reqwest::{Client, Response, StatusCode};
use reqwest::header::HeaderName;
use serde_json::{from_str, from_value, json, Value, Map};

use crate::config::magic::{AUTH_HEADER_NAME, JSON_RPC_VERSION, MISSING_STRING, WWW_AUTH_HEADER_NAME};
use crate::structs::{BlockTemplate, Config, PayoutDTO, TransferResponse};

async fn make_daemon_rpc_request(config: &Config, body: &Value) -> Option<Value> {
    return make_rpc_request(body,
                            &config.daemon_rpc_url,
                            &config.daemon_rpc_user,
                            &config.daemon_rpc_password,
                            None).await;
}

async fn make_wallet_rpc_request(config: &Config, body: &Value) -> Option<Value> {
    return make_rpc_request(body,
                            &config.wallet_rpc_url,
                            &config.wallet_rpc_user,
                            &config.wallet_rpc_password,
                            None).await;
}

fn parse_www_auth_header(response: Response, context: AuthContext) -> Option<AuthorizationHeader> {
    let resp_headers = response.headers();
    println!("{:?}", resp_headers);
    if !resp_headers.contains_key(WWW_AUTH_HEADER_NAME) {
        return None;
    }
    let www_auth_header = resp_headers.get(WWW_AUTH_HEADER_NAME).unwrap();
    match digest_auth::parse(www_auth_header.to_str().unwrap_or(MISSING_STRING)) {
        Ok(mut www_auth_header) => {
            match AuthorizationHeader::from_prompt(&mut www_auth_header, &context) {
                Ok(auth_header) => {
                    return Some(auth_header);
                }
                Err(_) => {
                    return None;
                }
            }
        }
        Err(_) => {
            return None;
        }
    }
}

async fn make_rpc_request(body: &Value,
                          uri: &String,
                          username: &String,
                          password: &String,
                          auth_header: Option<AuthorizationHeader>) -> Option<Value> {
    let client = Client::new();
    let mut builder = client.post(uri)
        .body(body.to_string());
    if !auth_header.is_none() {
        builder = builder.header(AUTH_HEADER_NAME, auth_header.unwrap())
    }
    let result = builder.send().await;
    if result.is_ok() {
        let response = result.ok().unwrap();
        if response.status() == StatusCode::OK {
            let resp_str = from_utf8(response.bytes().as_bytes()).unwrap_or(MISSING_STRING);
            if resp_str.eq(MISSING_STRING) || resp_str.is_empty() {
                info!("could not parse RPC response");
                return None;
            }
            let default = json!(MISSING_STRING);
            let parsed_val = serde_json::from_str(resp_str).unwrap_or(default);
            if parsed_val.as_str().unwrap_or(MISSING_STRING).eq(MISSING_STRING) {
                info!("could not parse RPC response");
                return None;
            }
            return Some(parsed_val);
        } else if response.status() == StatusCode::UNAUTHORIZED {
            if auth_header.is_some() {
                info!("already had one unauthorized RPC response");
                return None;
            }
            let mut context = AuthContext::new_post(username, password, uri, Some(body));
            match parse_www_auth_header(response, context) {
                Some(auth_header) => {
                    return make_rpc_request(body, uri, username, password, Some(auth_header));
                }
                None => {
                    return None;
                }
            }
        } else {
            info!("could not make RPC request");
            return None;
        }
    } else {
        info!("RPC call failed with error: {}", result.err().unwrap().to_string());
        return None;
    }
}

pub async fn get_latest_block_template(config: &Config) -> Option<BlockTemplate> {
    let body = json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": "0",
        "method": "get_block_template",
        "params": {
            "wallet_address": config.wallet,
            "reserve_size": config.pool_reserve_size_bytes
        }
    });

    let response = make_daemon_rpc_request(config, &body).await;
    if response.is_none() {
        warn!("could not get latest block template");
        return None;
    }
    let bt_val = from_value(response.unwrap());
    if bt_val.is_err() {
        warn!("could not parse latest block template");
        return None;
    }
    return Some(bt_val.unwrap());
}

pub async fn submit_block(config: &Config, block_hex: &str) -> Option<Value> {
    let body = json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": "0",
        "method": "submit_block",
        "params": [block_hex]
    });
    return make_daemon_rpc_request(config, &body).await;
}

pub async fn get_unlocked_balance(config: &Config) -> Option<u64> {
    let body = json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": "0",
        "method": "get_balance",
        "params": {
            "account_index": 0
        }
    });
    let req_result = make_wallet_rpc_request(config, &body).await;
    if req_result.is_none() {
        info!("could not get RPC balance result");
        return None;
    }
    let balance_response = req_result.unwrap();
    let balance_result = balance_response.get("result");
    if balance_result.is_none() {
        info!("could not get balance result");
        return None;
    }
    let unlocked_balance_result = balance_result.unwrap().get("unlocked_balance");
    if unlocked_balance_result.is_none() {
        info!("could not get unlocked balance result");
        return None;
    }
    let unlocked_balance_fmt_result = unlocked_balance_result.unwrap().as_u64();
    if unlocked_balance_fmt_result.is_none() {
        info!("could not parse unlocked balance result to u64");
        return None;
    }
    return Some(unlocked_balance_fmt_result.unwrap());
}

pub async fn submit_transfers(config: &Config,
                              accounts: &Vec<PayoutDTO>) -> Option<TransferResponse> {
    let destinations = accounts.iter().map(|x| x.for_transfer()).collect::<Vec<Value>>();
    let body = json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": "0",
        "method": "transfer_split",
        "params": {
            "account_index": 0,
            "destinations": destinations.as_slice(),
            "get_tx_metadata": true,
            "get_tx_hex": true,
            "get_tx_key": true,
            "unlock_time": 60,
            "priority": 0,
            "mixin": 10,
            "ring_size": 11,
            "new_algorithm": true
        }
    });
    let request_result_opt = make_wallet_rpc_request(config, &body).await;
    if request_result_opt.is_none() {
        warn!("could not make submit_block RPC request");
        return None;
    }
    let request_result = request_result_opt.unwrap();
    let transfer_response_result = request_result.get("result");
    if transfer_response_result.is_none() {
        warn!("could not make submit RPC request");
        return None;
    }
    let transfer_response = transfer_response_result.unwrap();
    let hash_list_result = transfer_response.get("tx_hash_list");
    if hash_list_result.is_none() {
        warn!("could not parse transaction hashes");
        return None;
    }
    let hash_list_val = hash_list_result.unwrap();
    let hash_list_vec_result = hash_list_val.as_array();
    if hash_list_vec_result.is_none() {
        warn!("could not cast transaction hashes to array");
        return None;
    }
    let hash_list = hash_list_vec_result.unwrap();
    if hash_list.is_empty() {
        warn!("no transaction hashes returned to parse");
        return None;
    }
    let tx_hashes = hash_list.iter().map(|x|
        x.as_str().unwrap_or(MISSING_STRING)).collect::<Vec<&str>>();
    if tx_hashes.iter().any(|x| x.eq(&MISSING_STRING)) {
        warn!("could not parse transaction hashes");
        return None;
    }
    let mut response: TransferResponse = Default::default();
    response.tx_hashes.extend(
        tx_hashes.iter().map(|x| x.to_string()).collect::<Vec<String>>());
    let keys_list_response = transfer_response.get("tx_keys_list");
    if keys_list_response.is_none() {
        warn!("could not parse transaction keys");
        return None;
    }
    let keys_list_vec_result = keys_list_response.unwrap().as_array();
    if hash_list_vec_result.is_none() {
        warn!("could not cast transaction keys to array");
        return None;
    }
    let keys_list_vec = keys_list_vec_result.unwrap();
    if keys_list_vec.is_empty() {
        warn!("no transaction keys returned to parse");
        return None;
    }
    let tx_keys = keys_list_vec.iter().map(|x|
        x.as_str().unwrap_or(MISSING_STRING)).collect::<Vec<&str>>();
    if tx_keys.iter().any(|x| x.eq(&MISSING_STRING)) {
        warn!("could not parse transaction keys");
        return None;
    }
    response.tx_keys.extend(
        tx_keys.iter().map(|x| x.to_string()).collect::<Vec<String>>());
    return Some(response);
}


