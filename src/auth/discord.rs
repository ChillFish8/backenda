use std::collections::HashMap;
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use reqwest::StatusCode;

static REDIRECT_URI: &str = "http://127.0.0.1:8000/api/v0/auth/authorize";
static DISCORD_URL: &str = "https://discord.com/api/v9";

lazy_static! {
    static ref CLIENT_ID: u64 = {
        std::env::var("DISCORD_CLIENT_ID")
            .map(|v| v.parse::<u64>().unwrap_or(0))
            .unwrap_or(0)
    };

    static ref CLIENT_SECRET: String = {
        std::env::var("DISCORD_CLIENT_SECRET")
            .unwrap_or_else(|_| "".to_string())
    };
}

#[derive(Serialize)]
pub struct ExchangeForm<'a> {
    client_id: u64,
    client_secret: &'a str,
    grant_type: &'static str,
    code: &'a str,
    redirect_uri: &'static str,
}

#[derive(Deserialize)]
pub struct ExchangeResp {
    access_token: String,

    #[serde(flatten)]
    _other: HashMap<String, Value>,
}

pub async fn exchange_code(code: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let body = ExchangeForm {
        client_id: *CLIENT_ID,
        client_secret: &CLIENT_SECRET,
        grant_type: "authorization_code",
        code,
        redirect_uri: REDIRECT_URI,
    };

    let resp = client.post(&format!("{}/oauth2/token", DISCORD_URL))
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&body)
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        let body: ExchangeResp = resp.json().await?;
        return Ok(body.access_token)
    }

    let status = resp.status().to_string();
    let body: serde_json::Value = resp.json().await?;
    error!(
        "error from discord with status {} body={}",
        status,
        serde_json::to_string_pretty(&body).unwrap(),
    );

    Err(Error::msg("an internal server error has arisen while talking to Discord."))
}

#[derive(Deserialize)]
pub struct UserInfo {
    #[serde(with = "discord_id")]
    pub id: i64,

    pub username: String,
    pub avatar: Option<String>,

    #[serde(flatten)]
    _other: Value,
}

pub async fn fetch_user_info(token: &str) -> Result<UserInfo> {
    let client = reqwest::Client::new();

    let resp = client.get(&format!("{}/users/@me", DISCORD_URL))
        .header(reqwest::header::AUTHORIZATION, bearer(token))
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        return Ok(resp.json().await?)
    }

    let status = resp.status().to_string();
    let body: serde_json::Value = resp.json().await?;
    error!(
        "error from discord with status {} body={}",
        status,
        serde_json::to_string_pretty(&body).unwrap(),
    );

    Err(Error::msg("an internal server error has arisen while talking to Discord."))
}

#[derive(Deserialize)]
pub struct Guild {
    #[serde(with = "discord_id")]
    pub id: i64,

    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,

    #[serde(with = "discord_id")]
    pub permissions: i64,

    #[serde(flatten)]
    _other: Value,
}

impl Guild {
    pub fn is_manager(&self) -> bool {
        self.owner
        | (self.permissions & (1 << 5) != 0)
        | (self.permissions & (1 << 3) != 0)
    }
}

pub async fn fetch_user_guilds(token: &str) -> Result<Vec<Guild>> {
    let client = reqwest::Client::new();

    let resp = client.get(&format!("{}/users/@me/guilds", DISCORD_URL))
        .header(reqwest::header::AUTHORIZATION, bearer(token))
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        return Ok(resp.json().await?)
    }

    let status = resp.status().to_string();
    let body: serde_json::Value = resp.json().await?;
    error!(
        "error from discord with status {} body={}",
        status,
        serde_json::to_string_pretty(&body).unwrap(),
    );

    Err(Error::msg("an internal server error has arisen while talking to Discord."))
}

fn bearer(token: &str) -> String {
    format!("Bearer {}", token)
}


mod discord_id {
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<i64>().map_err(serde::de::Error::custom)
    }
}