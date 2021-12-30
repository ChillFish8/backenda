use anyhow::anyhow;
use uuid::Uuid;
use poem_openapi::Object;
use scylla::{FromRow, IntoTypedRows};

use crate::db::Session;


#[derive(Object, FromRow)]
pub struct Playlist {
    id: Uuid,
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
