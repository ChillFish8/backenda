use poem_openapi::Object;
use anyhow::{anyhow, Result};
use scylla::IntoTypedRows;
use uuid::Uuid;

use crate::db::Session;
use super::user_info;

#[derive(Object)]
pub struct Playlist {
    id: Uuid,
    owner_id: String,
    title: String,
    description: Option<String>,
    items: Vec<Uuid>,
    nsfw: bool,
    is_public: bool,
    banner: Option<String>,
    votes: u32,
}

#[derive(Object)]
pub struct PlaylistEntry {
    id: Uuid,
    owner_id: String,
    title: String,
    description: Option<String>,
    ref_link: Option<String>,
    nsfw: bool,
    is_public: bool,
    votes: u32,
}

pub async fn get_playlists_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<Playlist>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        r#"
        SELECT id, title, description, items, nsfw, is_public, banner, votes
        FROM playlists
        WHERE owner_id = ?
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type PlaylistInfo = (Uuid, String, Option<String>, Vec<Uuid>, bool, bool, Option<String>, i32);

    let playlists = rows.into_typed::<PlaylistInfo>()
        .filter_map(|v| v.ok())
        .map(|info| Playlist {
            id: info.0,
            owner_id: user_id.to_string(),
            title: info.1,
            description: info.2,
            items: info.3,
            nsfw: info.4,
            is_public: info.5,
            banner: info.6,
            votes: info.7 as u32,
        })
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
        r#"
        SELECT id, title, description, ref_link, nsfw, is_public, votes
        FROM playlists_entries
        WHERE owner_id = ?
        "#,
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type EntryInfo = (Uuid, String, Option<String>, Option<String>, bool, bool, i32);

    let playlists = rows.into_typed::<EntryInfo>()
        .filter_map(|v| v.ok())
        .map(|info| PlaylistEntry {
            id: info.0,
            owner_id: user_id.to_string(),
            title: info.1,
            description: info.2,
            ref_link: info.3,
            nsfw: info.4,
            is_public: info.5,
            votes: info.6 as u32,
        })
        .collect();

    Ok(Some(playlists))
}