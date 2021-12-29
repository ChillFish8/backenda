use poem_openapi::Object;
use anyhow::{anyhow, Error, Result};
use scylla::IntoTypedRows;

use crate::db::Session;
use super::user_info;


#[derive(Object)]
pub struct Room {
    id: String,
    guild_id: Option<String>,
    owner_id: String,
    active_playlist: Option<String>,
    playing_now: Option<String>,
    title: String,
    topic: Option<String>,
    is_public: bool,
    invite_only: bool,
    active: bool,
    banner: Option<String>,
    created_on: i64,
}

pub async fn get_active_room_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Option<Room>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        r#"
        SELECT
            id,
            guild_id,
            owner_id,
            active_playlist,
            playing_now,
            title,
            topic,
            is_public,
            invite_only,
            banner,
            created_on
        FROM rooms
        WHERE owner_id = ? AND active = true;
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type RoomInfo = (
        String, Option<i64>, i64,
        Option<String>, Option<String>, String,
        Option<String>, bool, bool,
        Option<String>, chrono::Duration,
    );

    let info = match rows.into_typed::<RoomInfo>().next() {
        None => return Ok(Some(None)),
        Some(v) => v?,
    };

    Ok(Some(Some(Room {
        id: info.0,
        guild_id: info.1.map(|v| v.to_string()),
        owner_id: info.2.to_string(),
        active_playlist: info.3,
        playing_now: info.4,
        title: info.5,
        topic: info.6,
        is_public: info.7,
        invite_only: info.8,
        active: true,
        banner: info.9,
        created_on: info.10.num_seconds(),
    })))
}


pub async fn get_rooms_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<Room>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        r#"
        SELECT
            id,
            guild_id,
            owner_id,
            active_playlist,
            playing_now,
            title,
            topic,
            is_public,
            invite_only,
            banner,
            created_on,
            active
        FROM rooms
        WHERE owner_id = ?;
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type RoomInfo = (
        String, Option<i64>, i64,
        Option<String>, Option<String>, String,
        Option<String>, bool, bool,
        Option<String>, chrono::Duration, bool,
    );

    let rooms = rows.into_typed::<RoomInfo>()
        .filter_map(|v| v.ok())
        .map(|info| Room {
            id: info.0,
            guild_id: info.1.map(|v| v.to_string()),
            owner_id: info.2.to_string(),
            active_playlist: info.3,
            playing_now: info.4,
            title: info.5,
            topic: info.6,
            is_public: info.7,
            invite_only: info.8,
            banner: info.9,
            created_on: info.10.num_seconds(),
            active: info.11,
        })
        .collect();

    Ok(Some(rooms))
}