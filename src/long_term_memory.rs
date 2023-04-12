use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct IndexPayload {
    app: String,
    text: String,
}

#[derive(Serialize)]
pub struct SearchPayload {
    pub app: String,
    pub text: String,
}

pub async fn index(
    api_key: String,
    client_id: String,
    app: String,
    text: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.getmetal.io/v1/index";
    let payload = IndexPayload { app, text };
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("x-metal-api-key", HeaderValue::from_str(&api_key)?);
    headers.insert("x-metal-client-id", HeaderValue::from_str(&client_id)?);

    let _response = client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    Ok(())
}

pub async fn search(
    text: String,
    app: String,
    api_key: String,
    client_id: String,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let url = "https://api.getmetal.io/v1/search";
    let body = SearchPayload { app, text };
    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .json(&body)
        .header("Content-Type", "application/json")
        .header("x-metal-api-key", api_key)
        .header("x-metal-client-id", client_id)
        .send()
        .await?;

    let data: HashMap<String, Vec<HashMap<String, String>>> = response.json().await?;
    let results = data.get("data").cloned().unwrap_or_default();

    Ok(results)
}
