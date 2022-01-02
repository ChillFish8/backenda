use reqwest::{StatusCode, header};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;


lazy_static! {
    static ref SOCKETEER_URI: String = {
        std::env::var("SOCKETEER_URI")
            .unwrap_or_else(|_| "http://127.0.0.1:8800".to_string())
    };

    static ref SOCKETEER_KEY: String = {
        std::env::var("SOCKETEER_KEY")
            .unwrap_or_else(|_| "hello".to_string())
    };
}


#[derive(Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EventType {
    CandidateCall,
    CandidateAnswer,
    // PlaylistSelected,
    // TrackChange,
    // RoomClosed,
}


#[derive(Serialize)]
pub struct Event {
    room_id: Uuid,

    #[serde(rename = "type")]
    type_: EventType,

    data: Value,
}


pub async fn emit_event(room_id: Uuid, type_: EventType, data: Value) -> Result<()> {
    let event = Event {
        room_id,
        type_,
        data
    };

    let client = reqwest::Client::new();

    let resp = client.post(format!("{}/api/v0/emit", SOCKETEER_URI.as_str()))
        .json(&event)
        .header(header::AUTHORIZATION, format!("Bearer {}", SOCKETEER_KEY.as_str()))
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(anyhow!("Socketeer responded with bad status code {}", resp.status()))
    }
}