use std::time::Duration;

use log::{error, info};
use tokio::time::interval;
use tokio_postgres::Client as Db;

use crate::data::api::{get_accounts_for_payout, insert_block_template};
use crate::stream::http::{init_block_notify_listener, read_message};
use crate::stream::rpc::{get_latest_block_template, get_unlocked_balance, submit_transfers};
use crate::structs::Config;
use crate::util::is_production;

async fn update_block_template(config: &Config, db: &Db) -> bool {
    let mut block_template_result = get_latest_block_template(&config).await;
    if block_template_result.is_none() {
        error!("could not get latest block template");
        return false;
    }
    block_template_result = insert_block_template(db, &block_template_result.unwrap()).await;
    if block_template_result.is_none() {
        error!("could not insert block template");
        return false;
    }
    return true;
}

async fn process_payments(config: &Config, db: &Db) -> bool {
    let unlocked_balance_result = get_unlocked_balance(config).await;
    if unlocked_balance_result.is_none() {
        error!("could not get unlocked balance");
        return false;
    }
    let unlocked_balance = unlocked_balance_result.unwrap();
    let accounts_result = get_accounts_for_payout(db, config).await;
    if accounts_result.is_none() {
        error!("could not get accounts for payout");
        return true;
    }
    let accounts = accounts_result.unwrap();
    if accounts.is_empty() {
        info!("no accounts need payout");
        return true;
    }
    let total_payout_balance = accounts.iter().map(|a| a.balance as u64).collect::<Vec<u64>>().iter().sum();
    if unlocked_balance < total_payout_balance {
        error!("total payout balance: {} is greater than unlocked pool wallet balance: {}",
               total_payout_balance, unlocked_balance);
        return false;
    }
    let transfer_result = submit_transfers(config, &accounts).await;
    if transfer_result.is_none() {
        error!("unable to submit transfers");
        return true;
    }
    let transfer = transfer_result.unwrap();
    if transfer.tx_hashes.is_empty() || transfer.tx_keys.is_empty() {
        error!("failure to submit payment transfers");
        return false;
    }
    return true;
}

async fn process_shares(config: &Config, db: &Db) -> bool {
    return true;
}

pub async fn init_sync_chain_state_timer(config: &Config, db: &Db) {
    let period = Duration::from_secs(config.poll_rpc_interval_seconds as u64);
    let mut interval = interval(period);
    loop {
        interval.tick().await;
        if !update_block_template(config, db).await {
            error!("unable to update block template");
        }
    }
}

pub async fn listen_for_blocks(config: &Config, db: &Db) {
    let listener = init_block_notify_listener(config).await;
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        read_message(&stream).await;
        if !update_block_template(config, db).await {
            error!("unable to update block template");
        }
    }
}

pub async fn init_process_payments_timer(config: &Config, db: &Db) {
    if config.should_process_payments && !is_production() {
        panic!("payment processing not allowed yet");
    }
    if !config.should_process_payments {
        info!("payments processing external");
        return;
    }
    let period = Duration::from_secs(config.process_payments_timer_seconds as u64);
    let mut interval = interval(period);
    loop {
        interval.tick().await;
        if !process_shares(config, db).await {
            error!("could not process shares");
            continue;
        }
        if !process_payments(config, db).await {
            error!("could not process payments");
        }
    }
}
