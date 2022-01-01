use std::collections::HashMap;
use anyhow::anyhow;
use scylla::IntoTypedRows;
use poem_openapi::Object;

use crate::db::Session;
use crate::utils::JsSafeBigInt;

#[derive(Object)]
pub struct User {
    pub id: JsSafeBigInt,
    #[oai(skip)]
    pub access_servers: HashMap<i64, bool>,
    pub avatar: Option<String>,
    pub updated_on: i64,
    pub username: String,
}

#[derive(Object)]
pub struct Guild {
    pub id: JsSafeBigInt,
    pub icon: Option<String>,
    pub updated_on: i64,
    pub name: String,
    pub manager: bool,
}


/// Gets a user_id from the given access token if it's valid otherwise return None.
pub async fn get_user_id_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<i64>> {
    let result = sess.query_prepared(
        "SELECT user_id FROM access_tokens WHERE access_token = ?;",
        (token.to_string(),)
    ).await?;

    let user_id = match result.rows {
        None => None,
        Some(rows) => {
            if let Some(row) = rows.into_typed::<(i64,)>().next() {
                Some(row?.0)
            } else {
                None
            }
        }
    };

    Ok(user_id)
}


/// Gets a full user object from the given access token.
pub async fn get_user_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<User>> {
    let user_id = match get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    get_user_from_id(sess, user_id).await
}

pub async fn get_user_from_id(sess: &Session, user_id: i64) -> anyhow::Result<Option<User>> {
    let result = sess.query_prepared(
        "SELECT id, access_servers, avatar, updated_on, username FROM users WHERE id = ?;",
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type UserInfo = (JsSafeBigInt, HashMap<i64, bool>, Option<String>, chrono::Duration, String);

    let res = match rows.into_typed::<UserInfo>().next() {
        None => None,
        Some(v) => {
            let v = v?;
            Some(User {
                id: v.0,
                access_servers: v.1,
                avatar: v.2,
                updated_on: v.3.num_milliseconds(),
                username: v.4
            })
        },
    };

    Ok(res)
}

/// Gets the amount of credits the user currently has by a given token.
pub async fn get_vote_credits_for_token(sess: &Session, token: &str) -> anyhow::Result<Option<i32>> {
    let user_id = match get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    get_user_vote_credits(sess, user_id).await.map(|v| Some(v))
}

/// Gets the amount of credits the user currently has.
///
/// This assumes the user currently exists.
pub async fn get_user_vote_credits(sess: &Session, user_id: i64) -> anyhow::Result<i32> {
    let result = sess.query_prepared(
        "SELECT credits FROM user_vote_credits WHERE id = ?;",
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;


    let (credits,) = match rows.into_typed::<(i32,)>().next() {
        None => return Ok(0),
        Some(v) => v?,
    };

    Ok(credits)
}


/// Increments a user's vote credits by a given amount.
///
/// This assumes the user currently exists.
pub async fn adjust_user_credits(sess: &Session, user_id: i64, value: i32) -> anyhow::Result<()> {
    sess.query_prepared(
        "UPDATE user_vote_credits SET credit = credit + ? WHERE user_id = ?;",
        (value, user_id)
    ).await?;

    Ok(())
}


/// Gets all accessible guilds for a given access token.
pub async fn get_user_guilds_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<Vec<Guild>>> {
    let user = match get_user_from_token(sess, token).await? {
        None => return Ok(None),
        Some(u) => u,
    };

    let ids = user.access_servers.keys()
        .copied()
        .collect();

    let guilds = get_guilds(sess, ids).await?;

    let guilds = guilds.into_iter()
        .map(|v| Guild {
            id: v.0,
            icon:  v.1,
            updated_on:  v.2.num_milliseconds(),
            name:  v.3,
            manager: user.access_servers.get(&v.0).copied().unwrap_or(false)
        })
        .collect();

    Ok(Some(guilds))
}


type GuildInfo = (JsSafeBigInt, Option<String>, chrono::Duration, String);

/// Gets a guild based on it's guild id.
async fn get_guilds(sess: &Session, guild_ids: Vec<i64>) -> anyhow::Result<Vec<GuildInfo>> {
    let result = sess.query_prepared(
        "SELECT id, icon, updated_on, name FROM guilds WHERE id IN ?;",
        (guild_ids,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let guilds: Vec<GuildInfo> = rows.into_typed::<GuildInfo>()
        .filter_map(|v| v.ok())
        .collect();

    Ok(guilds)
}