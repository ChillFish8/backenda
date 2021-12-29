use anyhow::anyhow;
use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi, ApiResponse};
use poem_openapi::param::Query;
use poem_openapi::types::ToJSON;
use scylla::IntoTypedRows;
use serde_json::{json, Value};

use crate::utils::{JsonResponse, SuperUserBearer};
use crate::ApiTags;
use crate::db::Session;
use crate::users::notifications::{get_user_notifications, Icons, Notification};



#[derive(Object)]
pub struct NotificationCreation {
    #[oai(validator(minimum(value = "0")))]
    recipient_id: i64,

    #[oai(validator(max_length = 32, min_length = 2))]
    title: String,

    #[oai(validator(max_length = 256))]
    description: Option<String>,

    icon: Option<Icons>,
}

#[derive(Object)]
pub struct Created {
    id: String,
    username: String,
}

#[derive(ApiResponse)]
pub enum NotificationResponse<T: Send + Sync + ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 422)]
    NotFound(Json<Value>),
}


pub struct NotificationsApi;

#[OpenApi]
impl NotificationsApi {
    /// Create Notification
    ///
    /// Creates a notification for a user.
    #[oai(path = "/notifications", method = "post", tag = "ApiTags::Notifications")]
    pub async fn create_notification(
        &self,
        _token: SuperUserBearer,
        session: Data<&Session>,
        payload: Json<NotificationCreation>,
    ) -> Result<NotificationResponse<Created>> {

        let result = session.query_prepared(
            "SELECT username FROM users WHERE id = ?",
            (payload.0.recipient_id,)
        ).await?;

        let rows = result.rows
            .ok_or_else(|| anyhow!("expected returned rows"))?;


        let username = match rows.into_typed::<(String,)>().next() {
            None => return Ok(NotificationResponse::NotFound(Json(json!({
                "detail": "User does not exist with this id.",
                "user_id": payload.recipient_id,
            })))),
            Some(v) => v.map_err(anyhow::Error::from)?.0,
        };

        session.query(
            r#"
            INSERT INTO notifications (
                id,
                recipient_id,
                title,
                description,
                created_on,
                icon
            ) VALUES (uuid(), ?, ?, ?, toTimeStamp(now()), ?)"#,
            (
                payload.0.recipient_id,
                payload.0.title,
                payload.0.description,
                payload.0.icon.map(|v| v.to_string()),
                )
        ).await?;

        Ok(NotificationResponse::Ok(Json(Created {
            id: payload.0.recipient_id.to_string(),
            username
        })))
    }

    /// List Notifications
    ///
    /// Creates a notification for a user.
    #[oai(path = "/notifications", method = "get", tag = "ApiTags::Notifications")]
    pub async fn get_notification(
        &self,
        _token: SuperUserBearer,
        session: Data<&Session>,
        user_id: Query<i64>,
    ) -> Result<JsonResponse<Vec<Notification>>> {
        let notifications = get_user_notifications(&session, user_id.0).await?;
        Ok(JsonResponse::Ok(Json(notifications)))
    }
}
