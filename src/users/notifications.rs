use poem_openapi::Object;
use anyhow::{anyhow, Error, Result};
use scylla::IntoTypedRows;

use crate::db::Session;
use super::user_info;


#[derive(Object)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_on: i64,
    pub icon: Option<String>,
}


pub async fn get_user_notifications_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<Notification>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        "SELECT id, title, description, icon, created_on FROM notifications WHERE recipient_id = ?;",
        (user_id,),
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type NotificationInfo = (i64, String, Option<String>, Option<String>, chrono::Duration);
    let rows: Vec<Notification> = rows.into_typed::<NotificationInfo>()
        .filter_map(|v| v.ok())
        .map(|v| Notification {
            id: v.0.to_string(),
            title: v.1,
            description: v.2,
            icon: v.3,
            created_on: v.4.num_seconds(),
        })
        .collect();

    Ok(Some(rows))
}


pub async  fn delete_user_notification(
    sess: &Session,
    token: &str,
    id: &str
) -> Result<Option<()>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    sess.query_prepared(
        "DELETE FROM notifications WHERE id = ? AND recipient_id = ?;",
        (id, user_id),
    ).await?;

    Ok(Some(()))
}
