use crate::models::{AppState, MotorheadError};
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use std::error::Error;
use std::sync::Arc;

pub async fn incremental_summarization(
    openai_client: Client,
    context: Option<String>,
    messages: Vec<String>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let messages_joined = messages.join("\n");
    // Taken from Langchain
    let prev_summary = context.as_deref().unwrap_or_default();
    let progresive_prompt = format!(
        r#"
        Progressively summarize the lines of conversation provided, adding onto the previous summary returning a new summary. If the lines are meaningless just return NONE

        EXAMPLE
        Current summary:
        The human asks what the AI thinks of artificial intelligence. The AI thinks artificial intelligence is a force for good.
        New lines of conversation:
        Human: Why do you think artificial intelligence is a force for good?
        AI: Because artificial intelligence will help humans reach their full potential.
        New summary:
        The human asks what the AI thinks of artificial intelligence. The AI thinks artificial intelligence is a force for good because it will help humans reach their full potential.
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

    Ok(completion.to_string())
}

pub async fn handle_compaction(
    session_id: String,
    state_clone: Arc<Arc<AppState>>,
    mut conn: redis::aio::ConnectionManager,
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
        .query_async(&mut conn)
        .await?;

    let new_context_result =
        incremental_summarization(state_clone.openai_client.clone(), context, messages).await;

    if let Err(ref error) = new_context_result {
        log::error!("Problem getting summary: {:?}", error);
        return Err(MotorheadError::IncrementalSummarizationError(
            error.to_string(),
        ));
    }

    let new_context = new_context_result.unwrap_or_default();

    let redis_pipe_response_result: Result<((), ()), redis::RedisError> = redis::pipe()
        .cmd("LTRIM")
        .arg(&*session_id)
        .arg(0)
        .arg(i64::try_from(half).unwrap())
        .cmd("SET")
        .arg(context_key)
        .arg(new_context)
        .query_async(&mut conn)
        .await;

    match redis_pipe_response_result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error executing the redis pipeline: {:?}", e);
            Err(MotorheadError::RedisError(e))
        }
    }
}
