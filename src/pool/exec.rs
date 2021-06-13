use futures::future::join_all;
use log::error;
use tokio_postgres::Client as Db;

use crate::constants::strings::{INIT_BLOCK_NOTIFY_LISTENER,
                                INIT_PROCESS_PAYMENTS_TIMER,
                                INIT_SYNC_CHAIN_STATE_TIMER,
                                INIT_WORKER_POOL_LISTENER,
                                NODE_ROLE_BACKEND,
                                NODE_ROLE_COMBINED,
                                NODE_ROLE_WORKER};
use crate::data::init::init_database;
use crate::pool::backend::{init_process_payments_timer,
                           init_sync_chain_state_timer,
                           listen_for_blocks};
use crate::pool::worker::init_worker_pool_listener;
use crate::structs::Config;

async fn run_task(task_name: &str, config: &Config, db: &Db) {
    if task_name.eq(INIT_SYNC_CHAIN_STATE_TIMER) {
        init_sync_chain_state_timer(config, db).await;
    } else if task_name.eq(INIT_BLOCK_NOTIFY_LISTENER) {
        listen_for_blocks(config, db).await;
    } else if task_name.eq(INIT_WORKER_POOL_LISTENER) {
        init_worker_pool_listener(config, db).await;
    } else if task_name.eq(INIT_PROCESS_PAYMENTS_TIMER) {
        init_process_payments_timer(config, db).await;
    } else {
        error!("task name not recognized: {}", task_name)
    }
}

pub async fn run_pool(config: &Config) {
    // init the database
    let db = &init_database(config).await;
    // to add the async functions to the tasks vector they have to be the same function,
    // so we do this "run_task" hack.
    let mut tasks = Vec::new();
    match config.node_role.as_str() {
        NODE_ROLE_BACKEND => {
            tasks.push(run_task(INIT_SYNC_CHAIN_STATE_TIMER, config, db));
            tasks.push(run_task(INIT_BLOCK_NOTIFY_LISTENER, config, db));
            tasks.push(run_task(INIT_PROCESS_PAYMENTS_TIMER, config, db));
        }
        NODE_ROLE_WORKER => {
            tasks.push(run_task(INIT_WORKER_POOL_LISTENER, config, db));
        }
        NODE_ROLE_COMBINED => {
            tasks.push(run_task(INIT_SYNC_CHAIN_STATE_TIMER, config, db));
            tasks.push(run_task(INIT_BLOCK_NOTIFY_LISTENER, config, db));
            tasks.push(run_task(INIT_PROCESS_PAYMENTS_TIMER, config, db));
            tasks.push(run_task(INIT_WORKER_POOL_LISTENER, config, db));
        }
        _ => {
            panic!("node role not recognized: {}", config.node_role);
        }
    }
    // run all tasks concurrently
    join_all(tasks).await;
}
