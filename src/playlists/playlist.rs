use anyhow::anyhow;
use uuid::Uuid;
use poem_openapi::Object;
use scylla::{FromRow, IntoTypedRows};

use crate::db::Session;
use crate::utils::JsSafeBigInt;


#[derive(Object, FromRow)]
pub struct Playlist {
    pub id: Uuid,
    pub owner_id: JsSafeBigInt,
    pub banner: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub items: Vec<Uuid>,
    pub nsfw: bool,
    pub title: String,
    pub votes: i32,
}


pub async fn get_playlist_by_id(sess: &Session, id: Uuid) -> anyhow::Result<Option<Playlist>> {
    let result = sess.query_prepared(
        r#"
        SELECT * FROM playlists WHERE id = ?;
        "#,
        (id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;


    let playlist = match rows.into_typed::<Playlist>().next() {
        None => return Ok(None),
        Some(v) => v?,
    };

    Ok(Some(playlist))
}


pub async fn upvote_playlist(sess: &Session, user_id: i64, playlist_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "INSERT INTO playlist_votes (user_id, playlist_id) VALUES (?, ?);",
        (user_id, playlist_id.clone()),
    ).await?;

    sess.query_prepared(
        "UPDATE playlists SET votes = votes + 1 WHERE id = ?;",
        (playlist_id,)
    ).await?;

    Ok(())
}

pub async fn has_user_voted(sess: &Session, user_id: i64, playlist_id: Uuid) -> anyhow::Result<bool> {
    let result = sess.query_prepared(
        "SELECT true FROM playlist_votes WHERE user_id = ? AND playlist_id = ?;",
        (user_id, playlist_id)
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

pub async fn remove_playlist(sess: &Session, playlist_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "DELETE FROM playlists WHERE playlist_id = ?;",
        (playlist_id,)
    ).await?;

    sess.query_prepared(
        "DELETE FROM playlist_votes WHERE playlist_id = ?;",
        (playlist_id,)
    ).await?;

    Ok(())
}