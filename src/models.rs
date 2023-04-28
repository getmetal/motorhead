use redis::{FromRedisValue, RedisError, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub window_size: i64,
    pub session_cleanup: Arc<Mutex<HashMap<String, bool>>>,
    pub openai_client: async_openai::Client,
    pub long_term_memory: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SearchPayload {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemoryMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct MemoryMessagesAndContext {
    pub messages: Vec<MemoryMessage>,
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct MemoryResponse {
    pub messages: Vec<MemoryMessage>,
    pub context: Option<String>,
    pub tokens: Option<i64>,
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

impl From<Box<dyn Error + Send + Sync>> for MotorheadError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        MotorheadError::IncrementalSummarizationError(error.to_string())
    }
}

impl From<RedisError> for MotorheadError {
    fn from(err: RedisError) -> Self {
        MotorheadError::RedisError(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisearchResult {
    pub role: String,
    pub content: String,
    pub dist: f64,
}

impl FromRedisValue for RedisearchResult {
    fn from_redis_value(v: &Value) -> redis::RedisResult<Self> {
        let values: Vec<String> = redis::from_redis_value(v)?;
        let mut content = String::new();
        let mut role = String::new();
        let mut dist = 0.0;

        for i in 0..values.len() {
            match values[i].as_str() {
                "content" => content = values[i + 1].clone(),
                "role" => role = values[i + 1].clone(),
                "dist" => dist = values[i + 1].parse::<f64>().unwrap_or(0.0),
                _ => continue,
            }
        }

        Ok(RedisearchResult {
            role,
            content,
            dist,
        })
    }
}

pub fn parse_redisearch_response(response: &Value) -> Vec<RedisearchResult> {
    match response {
        Value::Bulk(array) => {
            let mut results = Vec::new();
            let n = array.len();

            for i in 1..n {
                if let Value::Bulk(ref bulk) = array[i] {
                    if let Ok(result) =
                        RedisearchResult::from_redis_value(&Value::Bulk(bulk.clone()))
                    {
                        results.push(result);
                    }
                }
            }

            results
        }
        _ => vec![],
    }
}
