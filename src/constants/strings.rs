///////////////////////////////////////////////////////////////////////////////////////////////////
////                                 ENUM VALUE STRINGS                                        ////
///////////////////////////////////////////////////////////////////////////////////////////////////

// message status
pub const STATUS_OK: &str = "OK";
pub const STATUS_KEEPALIVE: &str = "KEEPALIVED";

// task name
pub const INIT_SYNC_CHAIN_STATE_TIMER: &str = "INIT_SYNC_CHAIN_STATE_TIMER";
pub const INIT_PROCESS_PAYMENTS_TIMER: &str = "INIT_PROCESS_PAYMENTS_TIMER";
pub const INIT_BLOCK_NOTIFY_LISTENER: &str = "INIT_BLOCK_NOTIFY_LISTENER";
pub const INIT_WORKER_POOL_LISTENER: &str = "INIT_WORKER_POOL_LISTENER";

// node role
pub const NODE_ROLE_BACKEND: &str = "BACKEND";
pub const NODE_ROLE_WORKER: &str = "WORKER";
pub const NODE_ROLE_COMBINED: &str = "COMBINED";

// worker message method
pub const MESSAGE_METHOD_LOGIN: &str = "login";
pub const MESSAGE_METHOD_BLOCK_TEMPLATE: &str = "block_template";
pub const MESSAGE_METHOD_SUBMIT: &str = "submit";
pub const MESSAGE_METHOD_KEEPALIVE: &str = "keepalived";

// log level
pub const LOG_LEVEL_TRACE: &str = "TRACE";
pub const LOG_LEVEL_DEBUG: &str = "DEBUG";
pub const LOG_LEVEL_INFO: &str = "INFO";
pub const LOG_LEVEL_WARNING: &str = "WARNING";
pub const LOG_LEVEL_ERROR: &str = "ERROR";
pub const LOG_LEVEL_OFF: &str = "OFF";

// log style
pub const LOG_STYLE_SYSTEMD: &str = "SYSTEMD";
pub const LOG_STYLE_RUSTPOOL: &str = "RUSTPOOL";

// job state
pub const JOB_STATE_CREATED: &str = "CREATED";
pub const JOB_STATE_FINISHED: &str = "FINISHED";
pub const JOB_STATE_PROCESSED: &str = "PROCESSED";
pub const JOB_STATE_ERROR: &str = "ERROR";
