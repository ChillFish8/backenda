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
        r#"
        SELECT * FROM playlist_entries WHERE id = ?;
        "#,
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