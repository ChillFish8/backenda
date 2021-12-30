use poem_openapi::Object;
use anyhow::{anyhow, Result};
use scylla::{IntoTypedRows, FromRow};
use uuid::Uuid;

use crate::db::Session;
use crate::utils::JsSafeBigInt;
use super::user_info;
use crate::playlists::{PlaylistEntry, Playlist};


pub async fn get_playlists_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<Playlist>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        "SELECT * FROM playlists WHERE owner_id = ?",
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let playlists = rows.into_typed::<Playlist>()
        .filter_map(|v| v.ok())
        .collect();

    Ok(Some(playlists))
}


pub async fn get_playlist_entries_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<PlaylistEntry>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        "SELECT * FROM playlists_entries WHERE owner_id = ?",
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let playlists = rows.into_typed::<PlaylistEntry>()
        .filter_map(|v| v.ok())
        .collect();

    Ok(Some(playlists))
}