use async_openai::{
    config::{AzureConfig, OpenAIConfig},
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs,
        CreateChatCompletionResponse, CreateEmbeddingRequestArgs, Role,
    },
    Client,
};
use async_trait::async_trait;
use deadpool::managed::{Manager, RecycleResult};
use futures_util::future::try_join_all;
use redis::{FromRedisValue, RedisError, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct OpenAIClientManager {}

#[async_trait]
impl Manager for OpenAIClientManager {
    type Type = AnyOpenAIClient;
    type Error = MotorheadError;

    async fn create(&self) -> Result<AnyOpenAIClient, MotorheadError> {
        let openai_client = match (
            env::var("AZURE_API_KEY"),
            env::var("AZURE_DEPLOYMENT_ID"),
            env::var("AZURE_DEPLOYMENT_ID_ADA"),
            env::var("AZURE_API_BASE"),
        ) {
            (
                Ok(azure_api_key),
                Ok(azure_deployment_id),
                Ok(azure_deployment_id_ada),
                Ok(azure_api_base),
            ) => {
                let config = AzureConfig::new()
                    .with_api_base(&azure_api_base)
                    .with_api_key(&azure_api_key)
                    .with_deployment_id(azure_deployment_id)
                    .with_api_version("2023-05-15");

                let config_ada = AzureConfig::new()
                    .with_api_base(&azure_api_base)
                    .with_api_key(&azure_api_key)
                    .with_deployment_id(azure_deployment_id_ada)
                    .with_api_version("2023-05-15");

                AnyOpenAIClient::Azure {
                    embedding_client: Client::with_config(config_ada),
                    completion_client: Client::with_config(config),
                }
            }
            _ => {
                let openai_api_base = env::var("OPENAI_API_BASE");

                if let Ok(openai_api_base) = openai_api_base {
                    let embedding_config = OpenAIConfig::default().with_api_base(&openai_api_base);
                    let completion_config = OpenAIConfig::default().with_api_base(&openai_api_base);

                    AnyOpenAIClient::OpenAI {
                        embedding_client: Client::with_config(embedding_config),
                        completion_client: Client::with_config(completion_config),
                    }
                } else {
                    AnyOpenAIClient::OpenAI {
                        embedding_client: Client::new(),
                        completion_client: Client::new(),
                    }
                }
            }
        };
        Ok(openai_client)
    }

    async fn recycle(&self, _: &mut AnyOpenAIClient) -> RecycleResult<MotorheadError> {
        Ok(())
    }
}

pub enum AnyOpenAIClient {
    Azure {
        embedding_client: Client<AzureConfig>,
        completion_client: Client<AzureConfig>,
    },
    OpenAI {
        embedding_client: Client<OpenAIConfig>,
        completion_client: Client<OpenAIConfig>,
    },
}

impl AnyOpenAIClient {
    pub async fn create_chat_completion(
        &self,
        model: &str,
        progresive_prompt: &str,
    ) -> Result<CreateChatCompletionResponse, OpenAIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model(model)
            .messages([ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(progresive_prompt)
                .build()?])
            .build()?;

        match self {
            AnyOpenAIClient::Azure {
                completion_client, ..
            } => completion_client.chat().create(request).await,
            AnyOpenAIClient::OpenAI {
                completion_client, ..
            } => completion_client.chat().create(request).await,
        }
    }

    pub async fn create_embedding(
        &self,
        query_vec: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, OpenAIError> {
        match self {
            AnyOpenAIClient::OpenAI {
                embedding_client, ..
            } => {
                let request = CreateEmbeddingRequestArgs::default()
                    .model("text-embedding-ada-002")
                    .input(query_vec)
                    .build()?;

                let response = embedding_client.embeddings().create(request).await?;
                let embeddings: Vec<_> = response
                    .data
                    .iter()
                    .map(|data| data.embedding.clone())
                    .collect();

                Ok(embeddings)
            }
            AnyOpenAIClient::Azure {
                embedding_client, ..
            } => {
                let tasks: Vec<_> = query_vec
                    .into_iter()
                    .map(|query| async {
                        let request = CreateEmbeddingRequestArgs::default()
                            .model("text-embedding-ada-002")
                            .input(vec![query])
                            .build()?;

                        embedding_client.embeddings().create(request).await
                    })
                    .collect();

                let responses: Result<Vec<_>, _> = try_join_all(tasks).await;

                match responses {
                    Ok(successful_responses) => {
                        let embeddings: Vec<_> = successful_responses
                            .into_iter()
                            .flat_map(|response| response.data.into_iter())
                            .map(|data| data.embedding)
                            .collect();

                        Ok(embeddings)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}

pub struct AppState {
    pub window_size: i64,
    pub session_cleanup: Arc<Mutex<HashMap<String, bool>>>,
    pub openai_pool: deadpool::managed::Pool<OpenAIClientManager>,
    pub long_term_memory: bool,
    pub model: String,
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

impl std::error::Error for MotorheadError {}

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

            for item in array.iter().take(n).skip(1) {
                if let Value::Bulk(ref bulk) = item {
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

#[derive(serde::Deserialize)]
pub struct NamespaceQuery {
    pub namespace: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct GetSessionsQuery {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_size")]
    pub size: usize,
    pub namespace: Option<String>,
}

fn default_page() -> usize {
    1
}

fn default_size() -> usize {
    10
}
