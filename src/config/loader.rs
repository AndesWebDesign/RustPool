use std::fs::read_to_string;

use crate::config::magic::{ENVIRONMENT_VARIABLE_PREFIX, HOST_MAX_SIZE, WALLET_MAX_SIZE};
use crate::structs::Config;
use crate::util::is_production;

fn load_config(config_file_path: &str) -> Config {
    let contents = read_to_string(config_file_path).expect("can't read config file");
    if config_file_path.ends_with(".json") {
        return serde_json::from_str(contents.as_str()).expect("can't parse config file");
    } else if config_file_path.ends_with(".toml") {
        return toml::from_str(contents.as_str()).expect("can't parse config file");
    } else if config_file_path.ends_with(".yaml") {
        return serde_yaml::from_str(contents.as_str()).expect("can't parse config file");
    } else {
        panic!("config file must be JSON, YAML, or TOML format");
    }
}

fn load_config_overrides(mut config: Config) -> Config {
    if config.threads <= 0 {
        config.threads = num_cpus::get() as u8;
    }
    return config;
}

fn load_environment_overrides(mut config: Config) -> Config {
    let mut env_config = config::Config::default();
    env_config.merge(config::Environment::with_prefix(ENVIRONMENT_VARIABLE_PREFIX)).unwrap();

    // String
    config.daemon_rpc_url = env_config.get("daemon_rpc_url")
        .unwrap_or(config.daemon_rpc_url).to_string();
    config.wallet_rpc_url = env_config.get("wallet_rpc_url")
        .unwrap_or(config.wallet_rpc_url).to_string();
    config.block_notify_host = env_config.get("block_notify_host")
        .unwrap_or(config.block_notify_host).to_string();
    config.wallet = env_config.get("wallet")
        .unwrap_or(config.wallet).to_string();
    config.fee_wallet = env_config.get("fee_wallet")
        .unwrap_or(config.fee_wallet).to_string();
    config.pool_listen_host = env_config.get("pool_listen_host")
        .unwrap_or(config.pool_listen_host).to_string();
    config.log_level = env_config.get("log_level")
        .unwrap_or(config.log_level).to_string();
    config.log_style = env_config.get("log_style")
        .unwrap_or(config.log_style).to_string();
    config.database_type = env_config.get("database_type")
        .unwrap_or(config.database_type).to_string();
    config.database_host = env_config.get("database_host")
        .unwrap_or(config.database_host).to_string();
    config.database_name = env_config.get("database_name")
        .unwrap_or(config.database_name).to_string();
    config.database_user = env_config.get("database_user")
        .unwrap_or(config.database_user).to_string();
    config.database_password = env_config.get("database_password")
        .unwrap_or(config.database_password).to_string();
    config.node_role = env_config.get("node_role")
        .unwrap_or(config.node_role).to_string();

    // u8
    config.threads = env_config.get("threads")
        .unwrap_or(config.threads.to_string())
        .parse::<u8>().unwrap_or(config.threads);
    config.rpc_timeout_seconds = env_config.get("rpc_timeout_seconds")
        .unwrap_or(config.rpc_timeout_seconds.to_string())
        .parse::<u8>().unwrap_or(config.rpc_timeout_seconds);
    config.database_connect_timeout_seconds = env_config.get("database_connect_timeout_seconds")
        .unwrap_or(config.database_connect_timeout_seconds.to_string())
        .parse::<u8>().unwrap_or(config.database_connect_timeout_seconds);
    config.max_open_jobs_to_block = env_config.get("max_open_jobs_to_block")
        .unwrap_or(config.max_open_jobs_to_block.to_string())
        .parse::<u8>().unwrap_or(config.max_open_jobs_to_block);
    config.max_error_jobs_to_block = env_config.get("max_error_jobs_to_block")
        .unwrap_or(config.max_error_jobs_to_block.to_string())
        .parse::<u8>().unwrap_or(config.max_error_jobs_to_block);


    // u16
    config.block_notify_port = env_config.get("block_notify_port")
        .unwrap_or(config.block_notify_port.to_string())
        .parse::<u16>().unwrap_or(config.block_notify_port);
    config.pool_listen_port = env_config.get("pool_listen_port")
        .unwrap_or(config.pool_listen_port.to_string())
        .parse::<u16>().unwrap_or(config.pool_listen_port);
    config.miner_expected_seconds_per_share = env_config.get("miner_expected_seconds_per_share")
        .unwrap_or(config.miner_expected_seconds_per_share.to_string())
        .parse::<u16>().unwrap_or(config.miner_expected_seconds_per_share);
    config.pool_reserve_size_bytes = env_config.get("pool_reserve_size_bytes")
        .unwrap_or(config.pool_reserve_size_bytes.to_string())
        .parse::<u16>().unwrap_or(config.pool_reserve_size_bytes);
    config.database_port = env_config.get("database_port")
        .unwrap_or(config.database_port.to_string())
        .parse::<u16>().unwrap_or(config.database_port);
    config.poll_rpc_interval_seconds = env_config.get("poll_rpc_interval_seconds")
        .unwrap_or(config.poll_rpc_interval_seconds.to_string())
        .parse::<u16>().unwrap_or(config.poll_rpc_interval_seconds);
    config.pool_stats_window_seconds = env_config.get("pool_stats_window_seconds")
        .unwrap_or(config.pool_stats_window_seconds.to_string())
        .parse::<u16>().unwrap_or(config.pool_stats_window_seconds);

    // f64
    config.pool_fee = env_config.get("pool_fee")
        .unwrap_or(config.pool_fee.to_string())
        .parse::<f64>().unwrap_or(config.pool_fee);

    // u64
    config.pool_min_difficulty = env_config.get("pool_min_difficulty")
        .unwrap_or(config.pool_min_difficulty.to_string())
        .parse::<u64>().unwrap_or(config.pool_min_difficulty);
    config.auto_payment_min_balance_atomic_units = env_config.get(
        "auto_payment_min_balance_atomic_units").unwrap_or(
        config.auto_payment_min_balance_atomic_units.to_string())
        .parse::<u64>().unwrap_or(config.auto_payment_min_balance_atomic_units);
    config.manual_payment_min_balance_atomic_units = env_config.get(
        "manual_payment_min_balance_atomic_units").unwrap_or(
        config.manual_payment_min_balance_atomic_units.to_string())
        .parse::<u64>().unwrap_or(config.manual_payment_min_balance_atomic_units);


    // bool
    config.allow_self_select = env_config.get("allow_self_select")
        .unwrap_or(config.allow_self_select.to_string())
        .parse::<bool>().unwrap_or(config.allow_self_select);
    config.rx_use_full_memory = env_config.get("rx_use_full_memory")
        .unwrap_or(config.rx_use_full_memory.to_string())
        .parse::<bool>().unwrap_or(config.rx_use_full_memory);
    config.rx_use_large_pages = env_config.get("rx_use_large_pages")
        .unwrap_or(config.rx_use_large_pages.to_string())
        .parse::<bool>().unwrap_or(config.rx_use_large_pages);
    config.rx_set_secure_flag = env_config.get("rx_set_secure_flag")
        .unwrap_or(config.rx_set_secure_flag.to_string())
        .parse::<bool>().unwrap_or(config.rx_set_secure_flag);
    config.should_process_payments = env_config.get("should_process_payments")
        .unwrap_or(config.should_process_payments.to_string())
        .parse::<bool>().unwrap_or(config.should_process_payments);
    config.should_do_automatic_payments = env_config.get("should_do_automatic_payments")
        .unwrap_or(config.should_do_automatic_payments.to_string())
        .parse::<bool>().unwrap_or(config.should_do_automatic_payments);

    return config;
}


fn assert_valid(config: &Config) {
    // TODO: remove when template self-select enabled
    if config.allow_self_select && is_production() {
        panic!("template self-select is not enabled for production use yet");
    }
    if config.threads <= 0 {
        panic!("number of threads is invalid: {}", config.threads);
    }
    if config.pool_listen_host.len() > HOST_MAX_SIZE {
        panic!("pool host is too long: {}", config.pool_listen_host);
    }
    if config.database_host.len() > HOST_MAX_SIZE {
        panic!("database host is too long: {}", config.database_host);
    }
    if config.wallet.len() > WALLET_MAX_SIZE {
        panic!("wallet address is too long: {}", config.wallet);
    }
    if config.fee_wallet.len() > WALLET_MAX_SIZE {
        panic!("fee wallet address is too long: {}", config.fee_wallet);
    }
    if config.pool_fee < 0.0 {
        panic!("pool fee is negative: {}", config.pool_fee);
    }
    if config.pool_fee > 1.0 {
        panic!("pool fee is greater than 1: {}", config.pool_fee);
    }
    if config.max_error_jobs_to_block < 1 {
        panic!("max error jobs to block must be positive: {}", config.max_error_jobs_to_block);
    }
    if config.max_open_jobs_to_block < 1 {
        panic!("max open jobs to block must be positive: {}", config.max_open_jobs_to_block);
    }
    if config.pool_min_difficulty < 1 {
        panic!("pool minimum difficulty must be positive: {}", config.pool_min_difficulty);
    }
    if !vec!["POSTGRES"].contains(&config.database_type.as_str()) {
        panic!("database type not supported: {}", config.database_type);
    }
    if !vec!["BACKEND", "WORKER", "COMBINED"].contains(&config.node_role.as_str()) {
        panic!("node role not supported: {}", config.node_role);
    }
}

pub fn init_config(config_file_path: &str) -> Config {
    let mut config: Config = Config::default();
    if !config_file_path.is_empty() {
        config = load_config(config_file_path);
    }
    config = load_environment_overrides(config);
    config = load_config_overrides(config);
    assert_valid(&config);
    return config;
}
