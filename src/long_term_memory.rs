use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use byteorder::{LittleEndian, WriteBytesExt};
use nanoid::nanoid;
use std::io::Cursor;

fn encode(fs: Vec<f32>) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    for f in fs {
        buf.write_f32::<LittleEndian>(f).unwrap();
    }
    buf.into_inner()
}

pub async fn index_messages(
    messages: Vec<String>,
    session_id: String,
    openai_client: Client,
    mut redis_conn: redis::aio::ConnectionManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-ada-002")
        .input(messages.clone())
        .build()?;

    let response = openai_client.embeddings().create(request).await?;

    // TODO add used tokens let tokens_used = response.usage.total_tokens;
    for data in response.data {
        let id = nanoid!();
        let key = format!("motorhead:{}", id);
        let vector = encode(data.embedding);

        redis::cmd("HSET")
            .arg(key)
            .arg("session")
            .arg(&session_id)
            .arg("vector")
            .arg(vector)
            .arg("message")
            .arg(&messages[data.index as usize])
            .query_async::<_, ()>(&mut redis_conn)
            .await?;
    }

    Ok(())
}

pub async fn search_messages(
