use crate::models::{parse_redisearch_response, AnyOpenAIClient, MemoryMessage, RedisearchResult};
use byteorder::{LittleEndian, WriteBytesExt};
use nanoid::nanoid;
use redis::Value;
use std::io::Cursor;

fn encode(fs: Vec<f32>) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    for f in fs {
        buf.write_f32::<LittleEndian>(f).unwrap();
    }
    buf.into_inner()
}

pub async fn index_messages(
    messages: Vec<MemoryMessage>,
    session_id: String,
    openai_client: &AnyOpenAIClient,
    mut redis_conn: redis::aio::ConnectionManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let contents: Vec<String> = messages.iter().map(|msg| msg.content.clone()).collect();
    let embeddings = openai_client.create_embedding(contents.clone()).await?;

    // TODO add used tokens let tokens_used = response.usage.total_tokens;
    for (index, embedding) in embeddings.iter().enumerate() {
        let id = nanoid!();
        let key = format!("motorhead:{}", id);
        let vector = encode(embedding.to_vec());

        redis::cmd("HSET")
            .arg(key)
            .arg("session")
            .arg(&session_id)
            .arg("vector")
            .arg(vector)
            .arg("content")
            .arg(&contents[index])
            .arg("role")
            .arg(&messages[index].role)
            .query_async::<_, ()>(&mut redis_conn)
            .await?;
    }

    Ok(())
}

pub async fn search_messages(
    query: String,
    session_id: String,
    openai_client: &AnyOpenAIClient,
    mut redis_conn: redis::aio::ConnectionManager,
) -> Result<Vec<RedisearchResult>, Box<dyn std::error::Error>> {
    let response = openai_client.create_embedding(vec![query]).await?;
    let embeddings = response[0].clone();
    let vector = encode(embeddings);
    let query = format!("@session:{{{}}}=>[KNN 10 @vector $V AS dist]", session_id);

    let values: Vec<Value> = redis::cmd("FT.SEARCH")
        .arg("motorhead")
        .arg(query)
        .arg("PARAMS")
        .arg("2")
        .arg("V")
        .arg(vector)
        .arg("RETURN")
        .arg("3")
        .arg("role")
        .arg("content")
        .arg("dist")
        .arg("SORTBY")
        .arg("dist")
        .arg("DIALECT")
        .arg("2")
        .query_async(&mut redis_conn)
        .await?;

    let array_value = redis::Value::Bulk(values);
    let results = parse_redisearch_response(&array_value);

    Ok(results)
}
