use tokio_postgres::{Client, NoTls};

use crate::data::fixtures::load_test_data;
use crate::data::sql::{CREATE_ACCOUNT_TABLE_SQL,
                       CREATE_BLOCK_TEMPLATE_TABLE_SQL,
                       CREATE_JOB_TABLE_SQL,
                       CREATE_MINER_TABLE_SQL,
                       CREATE_PAYMENT_TABLE_SQL,
                       TEST_SCHEMA_EXISTS_SQL};
use crate::structs::Config;
use crate::util::is_production;

async fn load_schema_idempotent(db: &Client) {
    let rows = db.query(TEST_SCHEMA_EXISTS_SQL, &[]).await.expect(
        "test schema exists query failed");
    if !rows.is_empty() {
        return;
    }
    let schema_sql_statements = vec![
        CREATE_ACCOUNT_TABLE_SQL,
        CREATE_MINER_TABLE_SQL,
        CREATE_BLOCK_TEMPLATE_TABLE_SQL,
        CREATE_JOB_TABLE_SQL,
        CREATE_PAYMENT_TABLE_SQL,
    ];
    for statement in schema_sql_statements {
        db.query(statement, &[]).await.expect("schema creation query failed");
    }
    if !is_production() {
        load_test_data(db).await;
    }
}

pub async fn init_database(config: &Config) -> Client {
    // Connect to the database.
    let connect_string =
        &*format!("host={} port={} dbname={} user={} password={} connect_timeout={}",
                  config.database_host,
                  config.database_port,
                  config.database_name,
                  config.database_user,
                  config.database_password,
                  config.database_connect_timeout_seconds);
    let (client, connection) =
        tokio_postgres::connect(connect_string, NoTls).await.expect(
            "failed to connect to database");
    // The connection object performs the actual communication with the database,
    // it is recommended to spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            panic!("database connection closed with error: {}", e);
        }
    });
    // load schema if not present
    load_schema_idempotent(&client).await;
    // return the client
    return client;
}


