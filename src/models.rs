use redis::RedisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub window_size: i64,
    pub session_cleanup: Arc<Mutex<HashMap<String, bool>>>,
    pub openai_client: async_openai::Client,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct MemoryMessages {
    pub messages: Vec<MemoryMessage>,
}

#[derive(Serialize)]
pub struct MemoryResponse {
    pub messages: Vec<MemoryMessage>,
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub now: u128,
}

#[derive(Serialize)]
pub struct AckResponse {
    pub status: &'static str,
}

#[derive(Debug)]
pub enum MotorheadError {
    RedisError(RedisError),
    IncrementalSummarizationError(String),
}

impl std::fmt::Display for MotorheadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MotorheadError::RedisError(e) => write!(f, "Redis error: {}", e),
            MotorheadError::IncrementalSummarizationError(e) => {
                write!(f, "Incremental summarization error: {}", e)
            }
        }
    }
}

impl From<RedisError> for MotorheadError {
    fn from(err: RedisError) -> Self {
        MotorheadError::RedisError(err)
    }
}
