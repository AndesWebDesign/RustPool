use log::error;
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};

use crate::config::magic::{MESSAGE_BUFFER_SIZE, MISSING_STRING};
use crate::structs::Config;

async fn init_listener(addr: String) -> TcpListener {
    let listener = TcpListener::bind(addr).await.unwrap();
    return listener;
}

pub async fn init_worker_listener(config: &Config) -> TcpListener {
    let addr = format!("{}:{}", config.pool_listen_host, config.pool_listen_port);
    return init_listener(addr).await;
}

pub async fn init_block_notify_listener(config: &Config) -> TcpListener {
    let addr = format!("{}:{}", config.block_notify_host, config.block_notify_port);
    return init_listener(addr).await;
}

pub async fn read_message(stream: &TcpStream) -> Value {
    let missing = json!(MISSING_STRING);
    if stream.readable().await.ok() == None {
        error!("stream not readable");
        return missing;
    };
    let mut buffer = vec![0; MESSAGE_BUFFER_SIZE];
    let mut message: String = String::new();
    let mut buffer_size: usize = stream.try_read(&mut buffer).unwrap_or(usize::MAX);
    if buffer_size == usize::MAX || buffer_size == 0 {
        error!("could not read stream into buffer");
        return missing;
    };
    while buffer_size == MESSAGE_BUFFER_SIZE {
        let message_chunk = String::from_utf8(buffer.clone()).unwrap_or(MISSING_STRING.to_string());
        if message_chunk.eq(MISSING_STRING) {
            error!("could not parse message buffer to utf8 string");
            return missing;
        };
        message = [message, message_chunk].join("");
        buffer_size = stream.try_read(&mut buffer).unwrap_or(usize::MAX);
        if buffer_size == usize::MAX || buffer_size == 0 {
            error!("could not read stream into buffer");
            return missing;
        };
    }
    if buffer_size > 0 {
        buffer.truncate(buffer_size);
        let message_chunk = String::from_utf8(buffer.clone()).unwrap_or(MISSING_STRING.to_string());
        if message_chunk.eq(MISSING_STRING) {
            error!("could not parse message buffer to utf8 string");
            return missing;
        };
        message = [message, message_chunk].join("");
    }
    let result_json = serde_json::from_str(message.as_str()).unwrap_or(missing);
    if result_json.to_string().eq(MISSING_STRING) || result_json.to_string().is_empty() {
        error!("could not parse message string to json");
    }
    return result_json;
}

pub async fn write_message(stream: &TcpStream, message: Value) -> bool {
    if stream.writable().await.ok() == None {
        error!("stream not writable");
        return false;
    };
    let message_string = message.to_string();
    let message_array = message_string.as_bytes();
    let bytes_written = stream.try_write(message_array).unwrap_or(0);
    let success = message_array.len() == bytes_written;
    if !success {
        error!("only {} bytes written out of {}", bytes_written, message_array.len());
    }
    return success;
}
