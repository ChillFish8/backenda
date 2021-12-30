pub mod user_info;
pub mod notifications;
pub mod room_info;
pub mod playlist_info;

use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use poem_openapi::param::Query;
use serde_json::{json, Value};
use uuid::Uuid;

use user_info::{User, Guild};

use crate::ApiTags;
use crate::utils::{JsonResponse, TokenBearer};
use crate::db::Session;
use crate::playlists::{get_playlist_by_id, Playlist, PlaylistEntry};
use crate::rooms::models::Room;
use crate::users::notifications::Notification;


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

        if res.is_some() {
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
        match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => Ok(JsonResponse::Unauthorized),
            Some(room) =>  Ok(JsonResponse::Ok(Json(room))),
        }
    }

    /// Close User Active Room
    ///
    /// Closes the current user room if applicable.
    #[oai(path = "/users/@me/rooms/current", method = "delete", tag = "ApiTags::User")]
    pub async fn close_user_active_room(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let room = match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::Unauthorized),
            Some(room) =>  room,
        };

        match room {
            None => Ok(JsonResponse::Ok(Json(Value::Null))),
            Some(mut room) => {
                crate::rooms::set_room_inactive(&session, room.id.clone()).await?;

                room.active = false;

                Ok(JsonResponse::Ok(Json(Value::Null)))
            }
        }
    }

    /// Set Room Playlist
    ///
    /// Sets the user's active room playlist if applicable.
    #[oai(path = "/users/@me/rooms/playlist", method = "put", tag = "ApiTags::User")]
    pub async fn update_active_room_playlist(
        &self,
        playlist_id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Room>> {
        let playlist = match get_playlist_by_id(&session, playlist_id.0).await? {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "No playlist exists with this id."
            })))),
            Some(playlist) => playlist,
        };

        let room = match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::Unauthorized),
            Some(room) =>  room,
        };

        let mut room = match room {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "User has no active room."
            })))),
            Some(room) => room,
        };

        crate::rooms::set_room_playlist(&session, room.id.clone(), playlist.id).await?;

        room.active_playlist = Some(playlist.id);

        Ok(JsonResponse::Ok(Json(room)))
    }

    /// Set Room Now Playing
    ///
    /// Sets the user's active room playing now entry if applicable.
    #[oai(path = "/users/@me/rooms/entry", method = "put", tag = "ApiTags::User")]
    pub async fn update_active_room_active_entry(
        &self,
        entry_id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Room>> {
        let room = match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::Unauthorized),
            Some(room) =>  room,
        };

        let mut room = match room {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "User has no active room."
            })))),
            Some(room) => room,
        };

        let active_id = match room.active_playlist.clone() {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "No playlist selected."
            })))),
            Some(active_id) => active_id,
        };

        let playlist = match get_playlist_by_id(&session, active_id).await? {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "No playlist exists with this id."
            })))),
            Some(playlist) => playlist,
        };

        if !playlist.items.contains(&entry_id) {
            return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "No playlist entry exists for the current playlist."
            }))))
        }

        crate::rooms::set_room_currently_playing(
            &session,
            room.id.clone(),
            entry_id.0.clone(),
        ).await?;

        room.playing_now = Some(entry_id.0);

        Ok(JsonResponse::Ok(Json(room)))
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
        match room_info::get_rooms_for_token(&session, &token.0.token).await? {
            None => Ok(JsonResponse::Unauthorized),
            Some(rooms) =>  Ok(JsonResponse::Ok(Json(rooms))),
        }
    }

    /// Get User Playlists
    ///
    /// Get all user's playlists.
    #[oai(path = "/users/@me/playlists", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_playlists(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<Playlist>>> {
        match playlist_info::get_playlists_for_token(&session, &token.0.token).await? {
            None => Ok(JsonResponse::Unauthorized),
            Some(playlists) =>  Ok(JsonResponse::Ok(Json(playlists))),
        }
    }

    /// Get User Playlist Entries
    ///
    /// Get all user playlist entries.
    #[oai(path = "/users/@me/entries", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_entries(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<PlaylistEntry>>> {
        match playlist_info::get_playlist_entries_for_token(&session, &token.0.token).await? {
            None => Ok(JsonResponse::Unauthorized),
            Some(entries) =>  Ok(JsonResponse::Ok(Json(entries))),
        }
    }
}

