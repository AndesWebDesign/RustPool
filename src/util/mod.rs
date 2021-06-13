use std::env;

use hex::{decode, encode};
use rand::Rng;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::magic::MISSING_STRING;
use crate::structs::Config;

pub fn get_uuid_val_or_missing(obj: &Value, name: &str) -> Value {
    return obj.get(name).unwrap_or(&json!(Uuid::nil())).clone();
}

pub fn get_string_val_or_missing(obj: &Value, name: &str) -> Value {
    return obj.get(name).unwrap_or(&json!(MISSING_STRING)).clone();
}

pub fn get_i64_val_or_missing(obj: &Value, name: &str) -> Value {
    return obj.get(name).unwrap_or(&json!(0)).clone();
}

pub fn get_object_val_or_missing(obj: &Value, name: &str) -> Value {
    return obj.get(name).unwrap_or(&json!({})).clone();
}

pub fn is_production() -> bool {
    return env::var("RUSTPOOL_DEV").unwrap_or("".to_string()).is_empty();
}

pub fn generate_pool_nonce(config: &Config) -> String {
    let mut rng = rand::thread_rng();
    let mut pool_nonce: Vec<u8> = decode(config.pool_nonce_slug.as_str()).unwrap().clone();
    if pool_nonce.len() > (2 * config.pool_reserve_size_bytes) as usize {
        panic!("pool nonce slug is longer than template reserved space")
    }
    if pool_nonce.len() % 2 != 0 {
        panic!("pool nonce slug is not even length");
    }
    let remaining_range = (pool_nonce.len() / 2)..(config.pool_reserve_size_bytes as usize);
    for _ in remaining_range {
        pool_nonce.push(rng.gen::<u8>());
    }
    return encode(pool_nonce);
}
