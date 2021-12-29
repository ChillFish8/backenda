mod user_info;
mod notifications;
mod room_info;

use poem::web::Data;
use poem::Result;
use poem_openapi::auth::Bearer;
use poem_openapi::payload::Json;
use poem_openapi::{OpenApi, SecurityScheme, ApiResponse};
use poem_openapi::param::Query;
use poem_openapi::types::ToJSON;
use serde::Serialize;
use serde_json::Value;

use user_info::{User, Guild};

use crate::ApiTags;
use crate::db::Session;
use crate::users::notifications::Notification;
use crate::users::room_info::Room;

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
pub struct TokenBearer(Bearer);


#[derive(ApiResponse)]
pub enum JsonResponse<T: Send + Sync + ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 401)]
    Unauthorized,
}


pub struct UsersApi;

#[OpenApi]
impl UsersApi {
    /// Get User
    ///
    /// Get the user data associated with a give token.
    #[oai(path = "/users/@me", method = "get", tag = "ApiTags::User")]
    pub async fn get_user(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<User>> {
        if let Some(user) = user_info::get_user_from_token(&session, &token.0.token).await? {
            Ok(JsonResponse::Ok(Json(user)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }

    /// Get User Guilds
    ///
    /// Get the user guilds data associated with a give token.
    #[oai(path = "/users/@me/guilds", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_guilds(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<Guild>>> {
        if let Some(guilds) = user_info::get_user_guilds_from_token(&session, &token.0.token).await? {
            Ok(JsonResponse::Ok(Json(guilds)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }

    /// Get User Notifications
    ///
    /// Get the user's pending notifications.
    #[oai(path = "/users/@me/notifications", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_notifications(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<Notification>>> {
        let res = notifications::get_user_notifications_for_token(&session, &token.0.token).await?;
        if let Some(ns) = res {
            Ok(JsonResponse::Ok(Json(ns)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }

    /// Remove User Notification
    ///
    /// Removes a given notification from a user.
    #[oai(path = "/users/@me/notifications", method = "delete", tag = "ApiTags::User")]
    pub async fn remove_user_notifications(
        &self,
        id: Query<String>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let res = notifications::delete_user_notification(
            &session,
            &token.0.token,
            &id.0,
        ).await?;

        if res.is_none() {
            Ok(JsonResponse::Ok(Json(Value::Null)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }

    /// Get User Active Room
    ///
    /// Get the user's currently active room if applicable.
    #[oai(path = "/users/@me/rooms/current", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_active_room(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Option<Room>>> {
        if let Some(room) = room_info::get_active_room_for_token(&session, &token.0.token).await? {
            Ok(JsonResponse::Ok(Json(room)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }

    /// Get User Rooms
    ///
    /// Get all user rooms active or inactive.
    #[oai(path = "/users/@me/rooms", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_rooms(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<Room>>> {
        if let Some(rooms) = room_info::get_rooms_for_token(&session, &token.0.token).await? {
            Ok(JsonResponse::Ok(Json(rooms)))
        } else {
            Ok(JsonResponse::Unauthorized)
        }
    }
}

