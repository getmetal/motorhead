use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub window_size: i64,
    pub session_cleanup: Arc<Mutex<HashMap<String, bool>>>,
    pub openai_key: String,
    pub reduce_method: String,
    pub openai_client: openai_api::Client,
}

#[derive(Deserialize)]
pub struct MemoryMessage {
    pub message: String,
}

#[derive(Deserialize)]
pub struct MemoryMessages {
    pub messages: Vec<MemoryMessage>,
}

#[derive(Serialize)]
pub struct MemoryResponse {
    pub messages: Vec<String>,
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub now: u128,
}
