use redis::{self, RedisResult};

pub fn ensure_redisearch_index(
    redis: &redis::Client,
    vector_dimensions: usize,
    distance_metric: &str,
) -> RedisResult<()> {
    let mut con = redis.get_connection()?;
    let index_name = "motorhead";

    let index_info: Result<redis::Value, _> = redis::cmd("FT.INFO").arg(index_name).query(&mut con);

    if let Err(err) = index_info {
        if err
            .to_string()
            .to_lowercase()
            .contains("unknown: index name")
        {
            redis::cmd("FT.CREATE")
                .arg(index_name)
                .arg("ON")
                .arg("HASH")
                .arg("PREFIX")
                .arg("1")
                .arg("motorhead:")
                .arg("SCHEMA")
                .arg("session")
                .arg("TAG")
                .arg("content")
                .arg("TEXT")
                .arg("role")
                .arg("TEXT")
                .arg("vector")
                .arg("VECTOR")
                .arg("HNSW")
                .arg("6")
                .arg("TYPE")
                .arg("FLOAT32")
                .arg("DIM")
                .arg(vector_dimensions.to_string())
                .arg("DISTANCE_METRIC")
                .arg(distance_metric)
                .query(&mut con)?;
        } else {
            return Err(err);
        }
    }

    Ok(())
}
