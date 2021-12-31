use anyhow::anyhow;
use uuid::Uuid;
use poem_openapi::Object;
use scylla::{FromRow, IntoTypedRows};

use crate::db::Session;
use crate::utils::JsSafeBigInt;


#[derive(Object, FromRow)]
pub struct PlaylistEntry {
    pub id: Uuid,
    pub owner_id: JsSafeBigInt,
    pub description: Option<String>,
    pub is_public: bool,
    pub nsfw: bool,
    pub ref_link: Option<String>,
    pub title: String,
    pub votes: i32,
}


pub async fn get_entry_by_id(sess: &Session, id: Uuid) -> anyhow::Result<Option<PlaylistEntry>> {
    let result = sess.query_prepared(
        "SELECT * FROM playlist_entries WHERE id = ?;",
        (id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;


    let entry = match rows.into_typed::<PlaylistEntry>().next() {
        None => return Ok(None),
        Some(v) => v?,
    };

    Ok(Some(entry))
}

pub async fn get_entries_with_ids(sess: &Session, ids: Vec<Uuid>) -> anyhow::Result<Vec<PlaylistEntry>> {
    let result = sess.query_prepared(
        "SELECT * FROM playlist_entries WHERE id IN ?;",
        (ids,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let entries = rows.into_typed::<PlaylistEntry>()
        .filter_map(|v| v.ok())
        .collect();

    Ok(entries)
}

pub async fn upvote_playlist(sess: &Session, user_id: i64, entry_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "INSERT INTO playlist_entries_votes (user_id, entry_id) VALUES (?, ?);",
        (user_id, entry_id.clone()),
    ).await?;

    sess.query_prepared(
        "UPDATE playlist_entries_votes SET votes = votes + 1 WHERE id = ?;",
        (entry_id,)
    ).await?;

    Ok(())
}

pub async fn has_user_voted(sess: &Session, user_id: i64, entry_id: Uuid) -> anyhow::Result<bool> {
    let result = sess.query_prepared(
        "SELECT true FROM playlist_entries_votes WHERE user_id = ? AND entry_id = ?;",
        (user_id, entry_id)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    let has_votes = Option::flatten(
        rows.into_typed::<(bool,)>()
            .next()
            .map(|v| v.ok().map(|v| v.0))
    ).unwrap_or(false);

    Ok(has_votes)
}

pub async fn remove_entry(sess: &Session, entry_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "DELETE FROM playlists WHERE entry_id = ?;",
        (entry_id,)
    ).await?;

    sess.query_prepared(
        "DELETE FROM playlist_entries_votes WHERE entry_id = ?;",
        (entry_id,)
    ).await?;

    Ok(())
}