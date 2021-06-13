use tokio_postgres::Client;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::data::api::{create_job, get_job_for_miner, insert_block_template, login_miner};
use crate::structs::{BlockTemplate, Config, LoginMessage};

fn get_test_config() -> Config {
    let mut config: Config = Default::default();
    config.pool_stats_window_seconds = 100;
    config.pool_min_difficulty = 10000;
    return config;
}

fn get_login_message() -> LoginMessage {
    let mut message: LoginMessage = Default::default();
    message.client_id = Uuid::new_v4();
    message.host = "127.0.0.1".to_string();
    message.port = 1234;
    message.wallet = "\
46MshWSPBZdCPudVCcX7eZTE4qffChFiNhz3bF5Hmrib2eHjrn5i8aT2MTJaqnCVND1rjjGV3kdfcJLkJmeyzG\
DFRuPgyek".to_string();
    message.rigid = "worker1".to_string();
    return message;
}

fn get_block_template() -> BlockTemplate {
    let mut bt: BlockTemplate = Default::default();
    bt.blocktemplate_blob = "\
0e0eae8ecb8506fa5327c2bcae3d8876cae7de14994061e86e35a41e905fe6325297570b6184d000000000\
8b302520c3854e2fda42279694b5d71e9f988dcc0b4704ea5a98f95d11014fbf07".to_string();
    bt.blockhashing_blob = "\
0e0eae8ecb8506fa5327c2bcae3d8876cae7de14994061e86e35a41e905fe6325297570b6184d000000000\
02bee2900101ff82e2900101fcd58bc49b1d02dd77daa3dbdecf4bf724c7de39ed61244f98bc0227d74a48\
6765568cbe939e083e01db77eece456eb4345d47410ed9c0696f53ad916442e41af0bcc51edfbf13df1302\
1b6d696e65786d722e636f6d0c0c07000000000100000000000000000006232feb802efc4e76ceff7aeb9c\
bd9fa8fb8ade66a3722034c542ce61562135139ab74a03b9e92c876e44ad05f5f73d967e9a377373b58c02\
33e1060d20a0d473f70b33caf3a1af9fc40737a899c62dbc910197e996ce3f4e49444dc326f7b84549c1d5\
26212ba3b4999538c6d2b0dff52821b803bdace9ee9244c12007d8279be0f67ca5ede77b2a67e19590c959\
d1198664982fbac1c19c1110333a92bd219af029fadc56ea204fa5dcaaaf20bdb30c717cf5c4866aff9ea4\
4de9c6ef01390d".to_string();
    bt.reserved_size = 27;
    bt.reserved_offset = 140;
    bt.difficulty = 2371842;
    bt.height = 314261449893;
    bt.expected_reward = 1003822967548;
    bt.previous_hash = "fa5327c2bcae3d8876cae7de14994061e86e35a41e905fe6325297570b6184\
d0".to_string();
    bt.seed_hash = "b513e39fad73b944e2911e75638e884a07ef6ef0d5ce8569ba375a85e29cbe04".to_string();
    return bt;
}

pub async fn load_test_data(db: &Client) {
    let config = get_test_config();
    let msg = get_login_message();
    let bt = get_block_template();
    let miner = login_miner(db, &config, &msg).await.unwrap();
    insert_block_template(db, &bt).await;
    create_job(db, &config, &miner).await;
}
