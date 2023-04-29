use crate::models::{AppState, MotorheadError};
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use std::error::Error;
use std::sync::Arc;
use tiktoken_rs::p50k_base;

pub async fn incremental_summarization(
    openai_client: Client,
    context: Option<String>,
    mut messages: Vec<String>,
) -> Result<(String, u32), Box<dyn Error + Send + Sync>> {
    messages.reverse();
    let messages_joined = messages.join("\n");
    let prev_summary = context.as_deref().unwrap_or_default();
    // Taken from langchain
    let progresive_prompt = format!(
        r#"
Progressively summarize the lines of conversation provided, adding onto the previous summary returning a new summary. If the lines are meaningless just return NONE

EXAMPLE
Current summary:
The human asks who is the lead singer of Motörhead. The AI responds Lemmy Kilmister.
New lines of conversation:
Human: What are the other members of Motörhead?
AI: The original members included Lemmy Kilmister (vocals, bass), Larry Wallis (guitar), and Lucas Fox (drums), with notable members throughout the years including \"Fast\" Eddie Clarke (guitar), Phil \"Philthy Animal\" Taylor (drums), and Mikkey Dee (drums).
New summary:
The human asks who is the lead singer and other members of Motörhead. The AI responds Lemmy Kilmister is the lead singer and other original members include Larry Wallis, and Lucas Fox, with notable past members including \"Fast\" Eddie Clarke, Phil \"Philthy Animal\" Taylor, and Mikkey Dee.
END OF EXAMPLE

Current summary:
{prev_summary}
New lines of conversation:
{messages_joined}
New summary:
"#
    );
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content(progresive_prompt)
            .build()?])
        .build()?;

    let response = openai_client.chat().create(request).await?;

    let completion = response
        .choices
        .first()
        .ok_or("No completion found")?
        .message
        .content
        .clone();

    let usage = response.usage.ok_or("No Usage found")?;
    let tokens_used = usage.total_tokens;

    Ok((completion, tokens_used))
}

pub async fn handle_compaction(
    session_id: String,
    state_clone: Arc<Arc<AppState>>,
    mut redis_conn: redis::aio::ConnectionManager,
) -> Result<(), MotorheadError> {
    let half = state_clone.window_size / 2;
    let context_key = format!("{}_context", &*session_id);
    let (messages, mut context): (Vec<String>, Option<String>) = redis::pipe()
        .cmd("LRANGE")
        .arg(&*session_id)
        .arg(half)
        .arg(state_clone.window_size)
        .cmd("GET")
        .arg(context_key.clone())
        .query_async(&mut redis_conn)
        .await?;

    let max_tokens = 4096usize;
    let summary_max_tokens = 512usize;
    let buffer_tokens = 230usize;
    let max_message_tokens = max_tokens - summary_max_tokens - buffer_tokens;

    let mut total_tokens = 0;
    let mut temp_messages = Vec::new();
    let mut total_tokens_temp = 0;

    for message in messages {
        let bpe = p50k_base().unwrap();
        let message_tokens = bpe.encode_with_special_tokens(&message);
        let message_tokens_used = message_tokens.len();

        if total_tokens_temp + message_tokens_used <= max_message_tokens {
            temp_messages.push(message);
            total_tokens_temp += message_tokens_used;
        } else {
            let (summary, summary_tokens_used) = incremental_summarization(
                state_clone.openai_client.clone(),
                context.clone(),
                temp_messages,
            )
            .await?;

            total_tokens += summary_tokens_used;

            context = Some(summary);
            temp_messages = vec![message];
            total_tokens_temp = message_tokens_used;
        }
    }

    if !temp_messages.is_empty() {
        let (summary, summary_tokens_used) = incremental_summarization(
            state_clone.openai_client.clone(),
            context.clone(),
            temp_messages,
        )
        .await?;
        total_tokens += summary_tokens_used;
        context = Some(summary);
    }

    if let Some(new_context) = context {
        let token_count_key = format!("{}_tokens", &*session_id);
        let redis_pipe_response_result: Result<((), (), i64), redis::RedisError> = redis::pipe()
            .cmd("LTRIM")
            .arg(&*session_id)
            .arg(0)
            .arg(half)
            .cmd("SET")
            .arg(context_key)
            .arg(new_context)
            .cmd("INCRBY")
            .arg(token_count_key)
            .arg(total_tokens)
            .query_async(&mut redis_conn)
            .await;

        match redis_pipe_response_result {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Error executing the redis pipeline: {:?}", e);
                Err(MotorheadError::RedisError(e))
            }
        }
    } else {
        log::error!("No context found after summarization");
        Err(MotorheadError::IncrementalSummarizationError(
            "No context found after summarization".to_string(),
        ))
    }
}
