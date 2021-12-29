mod user_info;
mod notifications;

use poem::web::Data;
use poem::Result;
use poem_openapi::auth::Bearer;
use poem_openapi::payload::Json;
use poem_openapi::{OpenApi, SecurityScheme, ApiResponse};
use poem_openapi::param::Query;
use serde_json::Value;

use user_info::{User, Guild};

use crate::ApiTags;
use crate::db::Session;
use crate::users::notifications::Notification;

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
pub struct TokenBearer(Bearer);


#[derive(ApiResponse)]
pub enum GetUserResp {
    #[oai(status = 200)]
    Ok(Json<User>),

    #[oai(status = 401)]
    Unauthorized,
}

#[derive(ApiResponse)]
pub enum GetUserGuildsResp {
    #[oai(status = 200)]
    Ok(Json<Vec<Guild>>),

    #[oai(status = 401)]
    Unauthorized,
}

#[derive(ApiResponse)]
pub enum GetUserNotificationsResp {
    #[oai(status = 200)]
    Ok(Json<Vec<Notification>>),

    #[oai(status = 401)]
    Unauthorized,
}

#[derive(ApiResponse)]
pub enum RemoveNotificationResp {
    #[oai(status = 200)]
    Ok(Json<Value>),

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
    ) -> Result<GetUserResp> {
        if let Some(user) = user_info::get_user_from_token(&session, &token.0.token).await? {
            Ok(GetUserResp::Ok(Json(user)))
        } else {
            Ok(GetUserResp::Unauthorized)
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
    ) -> Result<GetUserGuildsResp> {
        if let Some(guilds) = user_info::get_user_guilds_from_token(&session, &token.0.token).await? {
            Ok(GetUserGuildsResp::Ok(Json(guilds)))
        } else {
            Ok(GetUserGuildsResp::Unauthorized)
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
    ) -> Result<GetUserNotificationsResp> {
        let res = notifications::get_user_notifications_for_token(&session, &token.0.token).await?;
        if let Some(ns) = res {
            Ok(GetUserNotificationsResp::Ok(Json(ns)))
        } else {
            Ok(GetUserNotificationsResp::Unauthorized)
        }
    }

    /// Remove User Notification
    ///
    /// Removes a given notification from a user.
    #[oai(path = "/users/@me/rooms", method = "delete", tag = "ApiTags::User")]
    pub async fn remove_user_notifications(
        &self,
        id: Query<String>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<RemoveNotificationResp> {
        let res = notifications::delete_user_notification(
            &session,
            &token.0.token,
            &id.0,
        ).await?;

        if res.is_none() {
            Ok(RemoveNotificationResp::Ok(Json(Value::Null)))
        } else {
            Ok(RemoveNotificationResp::Unauthorized)
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
    ) -> Result<GetUserGuildsResp> {
        todo!()
    }

    /// Get User Rooms
    ///
    /// Get all user rooms active or inactive.
    #[oai(path = "/users/@me/rooms", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_rooms(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<GetUserGuildsResp> {
        todo!()
    }
}

