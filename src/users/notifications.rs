use std::str::FromStr;
use poem_openapi::{Object, Enum};
use anyhow::{anyhow, Result};
use scylla::IntoTypedRows;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::db::Session;
use super::user_info;

#[derive(Enum, Display, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[oai(rename_all = "lowercase")]
pub enum Icons {
    News,
    Premium,
    Info,
    Coins,
    Issues,
    Discord,
}


#[derive(Object)]
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_on: i64,
    pub icon: Option<Icons>,
}


pub async fn get_user_notifications_for_token(
    sess: &Session,
    token: &str,
) -> Result<Option<Vec<Notification>>> {
    let user_id = match user_info::get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    get_user_notifications(sess, user_id).await.map(Some)
}

pub async fn get_user_notifications(sess: &Session, user_id: i64) -> Result<Vec<Notification>> {
    let result = sess.query_prepared(
        "SELECT id, title, description, icon, created_on FROM notifications WHERE recipient_id = ?;",
        (user_id,),
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type NotificationInfo = (Uuid, String, Option<String>, Option<String>, chrono::Duration);
    let rows: Vec<Notification> = rows.into_typed::<NotificationInfo>()
        .filter_map(|v| {
            v.ok()
        })
        .map(|v| Notification {
            id: v.0,
            title: v.1,
            description: v.2,
            icon: Option::flatten(v.3.map(|v| Icons::from_str(&v).ok())),
            created_on: v.4.num_milliseconds(),
        })
        .collect();

    Ok(rows)
}


pub async fn delete_user_notification(
    sess: &Session,
    token: &str,
    id: Uuid,
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
