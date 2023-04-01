use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use std::error::Error;

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
