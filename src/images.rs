use std::collections::HashMap;
use reqwest::StatusCode;
use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use serde_json::Value;


lazy_static! {
    static ref LUST_URI: String = {
        std::env::var("LUST_URI")
            .unwrap_or_else(|_| "http://127.0.0.1:7070".to_string())
    };
}


#[derive(Serialize)]
struct UploadPayload {
    format: String,
    data: String,
    category: String,
}

#[derive(Deserialize)]
struct UploadData {
    file_id: String,

    #[serde(flatten)]
    _other: HashMap<String, Value>
}

#[derive(Deserialize)]
struct UploadResponse {
    data: UploadData,

    #[serde(rename = "status")]
    _status: u16,
}


pub async fn fetch_and_upload(image_url: &str) -> anyhow::Result<Option<String>> {
    let client = reqwest::Client::new();

    let resp = client.get(image_url)
        .send()
        .await?;

    if resp.status() != StatusCode::OK {
        return Err(anyhow!("expected 200 OK response from image server, got {}", resp.status().as_u16()))
    }

    if let Some(size) = resp.content_length() {
        if size > 5_000_000 {  // reject if it's more than 5 MB
            return Ok(None)
        }
    } else {
        return Ok(None)
    }

    let format = match image_url.rsplit_once(".") {
        None => return Ok(None),
        Some((_, format)) => format
    };

    let data = resp.bytes().await?;
    let encoded = base64::encode(data);

    let payload = UploadPayload {
        format: format.to_string(),
        data: encoded,
        category: "banners".to_string(),
    };

    let resp: UploadResponse = client.post(format!("{}/admin/create/image", LUST_URI.as_str()))
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;


    Ok(Some(resp.data.file_id))
}