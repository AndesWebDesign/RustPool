use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::magic::{MISSING_STRING, WALLET_MAX_SIZE};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub threads: u8,
    pub daemon_rpc_url: String,
    pub daemon_rpc_user: String,
    pub daemon_rpc_password: String,
    pub wallet_rpc_url: String,
    pub wallet_rpc_user: String,
    pub wallet_rpc_password: String,
    pub block_notify_host: String,
    pub block_notify_port: u16,
    pub rpc_timeout_seconds: u8,
    pub wallet: String,
    pub fee_wallet: String,
    pub pool_listen_host: String,
    pub pool_listen_port: u16,
    pub pool_fee: f64,
    pub pool_min_difficulty: u64,
    pub pool_stats_window_seconds: u16,
    pub miner_expected_seconds_per_share: u16,
    pub pool_reserve_size_bytes: u16,
    pub pool_nonce_slug: String,
    pub max_error_jobs_to_block: u8,
    pub max_open_jobs_to_block: u8,
    pub log_level: String,
    pub log_style: String,
    pub database_type: String,
    pub database_host: String,
    pub database_port: u16,
    pub database_name: String,
    pub database_user: String,
    pub database_password: String,
    pub database_connect_timeout_seconds: u8,
    pub node_role: String,
    pub poll_rpc_interval_seconds: u16,
    pub should_process_payments: bool,
    pub should_do_automatic_payments: bool,
    pub process_payments_timer_seconds: u16,
    pub auto_payment_min_balance_atomic_units: u64,
    pub manual_payment_min_balance_atomic_units: u64,
    pub allow_self_select: bool,
    pub rx_use_full_memory: bool,
    pub rx_use_large_pages: bool,
    pub rx_set_secure_flag: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub blockhashing_blob: String,
    pub blocktemplate_blob: String,
    pub seed_hash: String,
    pub next_seed_hash: Option<String>,
    pub reserved_offset: i32,
    pub reserved_size: i32,
    pub difficulty: i64,
    pub height: i64,
    pub expected_reward: i64,
    pub previous_hash: String,
    pub origin: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct JobDTO {
    pub job_id: Uuid,
    pub state: String,
    pub pool_nonce: String,
    pub blockhashing_blob: String,
    pub blocktemplate_blob: String,
    pub seed_hash: String,
    pub next_seed_hash: Option<String>,
    pub height: i64,
    pub previous_hash: String,
    pub reserved_offset: i32,
    pub difficulty: i64,
    pub target: i64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct MinerDTO {
    pub id: i64,
    pub client_id: Uuid,
    pub host: String,
    pub port: i32,
    pub wallet: String,
    pub rigid: String,
    pub banned: bool,
    pub all_jobs: i32,
    pub open_jobs: i32,
    pub error_jobs: i32,
}

impl MinerDTO {
    pub fn can_have_job(&self, config: &Config) -> bool {
        if self.banned {
            warn!("miner is banned");
            return false;
        }
        if self.error_jobs > config.max_error_jobs_to_block as i32 {
            warn!("miner has too many error jobs");
            return false;
        }
        if self.open_jobs >= config.max_open_jobs_to_block as i32 {
            warn!("miner has too many open jobs");
            return false;
        }
        return true;
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct LoginMessage {
    pub host: String,
    pub port: i32,
    pub message_id: String,
    pub mode: String,
    pub client_id: Uuid,
    pub wallet: String,
    pub rigid: String,
}

impl LoginMessage {
    pub fn is_valid(&self) -> bool {
        let required_string_fields = vec![
            &self.host,
            &self.message_id,
            &self.wallet,
            &self.rigid
        ];
        if required_string_fields.iter().any(|&x| x.eq(MISSING_STRING) || x.eq("")) {
            println!("req");
            return false;
        }
        if self.port <= 0 {
            return false;
        }
        if self.wallet.len() > WALLET_MAX_SIZE {
            warn!("wallet too long");
            return false;
        }
        return true;
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct MinerTemplateMessage {
    pub host: String,
    pub port: i32,
    pub message_id: String,
    pub client_id: Uuid,
    pub job_id: Uuid,
    pub blob: String,
    pub height: i64,
    pub difficulty: i64,
    pub prev_hash: String,
}

impl MinerTemplateMessage {
    pub fn is_valid(&self) -> bool {
        let required_string_fields = vec![
            &self.host,
            &self.message_id,
            &self.blob,
            &self.prev_hash,
        ];
        if required_string_fields.iter().any(|&x| x.eq(MISSING_STRING) || x.eq("")) {
            return false;
        }
        if self.client_id.is_nil() {
            return false;
        }
        if self.job_id.is_nil() {
            return false;
        }
        if self.port <= 0 {
            return false;
        }
        if self.height <= 0 {
            return false;
        }
        if self.difficulty <= 0 {
            return false;
        }
        return true;
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SubmitBlockMessage {
    pub host: String,
    pub port: i32,
    pub message_id: String,
    pub client_id: Uuid,
    pub job_id: Uuid,
    pub nonce: String,
    pub result: String,
}

impl SubmitBlockMessage {
    pub fn is_valid(&self) -> bool {
        let required_string_fields = vec![
            &self.host,
            &self.message_id,
            &self.result,
            &self.nonce
        ];
        if required_string_fields.iter().any(|&x| x.eq(MISSING_STRING) || x.eq("")) {
            return false;
        }
        if self.client_id.is_nil() {
            return false;
        }
        if self.job_id.is_nil() {
            return false;
        }
        if self.port <= 0 {
            return false;
        }
        return true;
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct PayoutDTO {
    pub id: i64,
    pub address: String,
    pub balance: i64,
}

impl PayoutDTO {
    pub fn for_transfer(&self) -> Value {
        return json!({
            "address": self.address,
            "amount": self.balance,
        });
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct TransferResponse {
    pub tx_hashes: Vec<String>,
    pub tx_keys: Vec<String>,
}
