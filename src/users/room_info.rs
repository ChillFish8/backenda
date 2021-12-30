use anyhow::{anyhow, Result};
use scylla::IntoTypedRows;

use crate::db::Session;
use crate::models::{Room, RoomInfo, RoomInfoNoActive};
use super::user_info;


pub async fn get_active_room_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Option<Room>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    get_active_room_for_user_id(sess, user_id).await.map(|v| Some(v))
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
            active
        FROM rooms
        WHERE owner_id = ?;
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let rooms = rows.into_typed::<RoomInfo>()
        .filter_map(|v| v.ok())
        .map(|v| Room::from(v))
        .collect();

    Ok(Some(rooms))
}


pub async fn get_active_room_for_user_id(sess: &Session, user_id: i64) -> Result<Option<Room>> {
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
            banner
        FROM rooms
        WHERE owner_id = ? AND active = true;
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;


    let info = match rows.into_typed::<RoomInfoNoActive>().next() {
        None => return Ok(None),
        Some(v) => v?,
    };

    Ok(Some(Room::from(info)))
}