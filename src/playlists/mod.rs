mod entries;

use anyhow::anyhow;
use uuid::Uuid;
use poem_openapi::Object;
use scylla::{FromRow, IntoTypedRows};

pub use entries::*;
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
