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
) -> Result<(String, usize), Box<dyn Error + Send + Sync>> {
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

    let bpe = p50k_base().unwrap();
    let tokens_used = bpe.encode_with_special_tokens(&progresive_prompt);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful AI assistant.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(progresive_prompt)
                .build()?,
        ])
        .build()?;

    let response = openai_client.chat().create(request).await?;

    let completion = response
        .choices
        .first()
        .ok_or("No completion found")?
        .message
        .content
        .clone();

    Ok((completion.to_string(), tokens_used.len()))
}

pub async fn handle_compaction(
    session_id: String,
    state_clone: Arc<Arc<AppState>>,
    mut redis_conn: redis::aio::ConnectionManager,
) -> Result<(), MotorheadError> {
    let half = state_clone.window_size / 2;
    let context_key = format!("{}_context", &*session_id);
    let (messages, context): (Vec<String>, Option<String>) = redis::pipe()
        .cmd("LRANGE")
        .arg(&*session_id)
        .arg(i64::try_from(half).unwrap())
        .arg(i64::try_from(state_clone.window_size).unwrap())
        .cmd("GET")
        .arg(context_key.clone())
        .query_async(&mut redis_conn)
        .await?;

    let new_context_result =
        incremental_summarization(state_clone.openai_client.clone(), context, messages).await;

    if let Err(ref error) = new_context_result {
        log::error!("Problem getting summary: {:?}", error);
        return Err(MotorheadError::IncrementalSummarizationError(
            error.to_string(),
        ));
    }

    let (new_context, tokens_used) = new_context_result.unwrap_or_default();

    let token_count_key = format!("{}_tokens", &*session_id);
    let redis_pipe_response_result: Result<((), (), i64), redis::RedisError> = redis::pipe()
        .cmd("LTRIM")
        .arg(&*session_id)
        .arg(0)
        .arg(i64::try_from(half).unwrap())
        .cmd("SET")
        .arg(context_key)
        .arg(new_context)
        .cmd("INCRBY")
        .arg(token_count_key)
        .arg(tokens_used)
        .query_async(&mut redis_conn)
        .await;

    match redis_pipe_response_result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error executing the redis pipeline: {:?}", e);
            Err(MotorheadError::RedisError(e))
        }
    }
}
