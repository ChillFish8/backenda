pub mod user_info;
pub mod notifications;
pub mod room_info;
pub mod playlist_info;

use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};
use poem_openapi::param::Query;
use serde_json::Value;
use uuid::Uuid;

use user_info::{User, Guild};

use crate::ApiTags;
use crate::utils::{JsonResponse, SuperUserBearer, TokenBearer};
use crate::db::Session;
use crate::playlists::{get_playlist_by_id, Playlist, PlaylistEntry};
use crate::rooms::models::{ArchivedRoom, Room};
use crate::users::notifications::Notification;


#[derive(Object)]
pub struct CreditResponse {
    credits: i32,
}

pub struct UsersApi;

#[OpenApi]
impl UsersApi {
    /// Get User
    ///
    /// Get the user data associated with a given token.
    #[oai(path = "/users/@me", method = "get", tag = "ApiTags::User")]
    pub async fn get_user(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<User>> {
        if let Some(user) = user_info::get_user_from_token(&session, &token.0.token).await? {
            Ok(JsonResponse::ok(user))
        } else {
            Ok(JsonResponse::unauthorized())
        }
    }

    /// Get User Credits
    ///
    /// Get the user voting credits associated with a given token.
    #[oai(path = "/users/@me/credits", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_credits(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<CreditResponse>> {
        if let Some(credits) = user_info::get_vote_credits_for_token(&session, &token.0.token).await? {
            Ok(JsonResponse::ok(CreditResponse { credits }))
        } else {
            Ok(JsonResponse::unauthorized())
        }
    }

    /// Get User Guilds
    ///
    /// Get the user guilds data associated with a given token.
    #[oai(path = "/users/@me/guilds", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_guilds(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<Guild>>> {
        if let Some(guilds) = user_info::get_user_guilds_from_token(&session, &token.0.token).await? {
            Ok(JsonResponse::ok(guilds))
        } else {
            Ok(JsonResponse::unauthorized())
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
            Ok(JsonResponse::ok(ns))
        } else {
            Ok(JsonResponse::unauthorized())
        }
    }

    /// Remove User Notification
    ///
    /// Removes a given notification from a user.
    #[oai(path = "/users/@me/notifications", method = "delete", tag = "ApiTags::User")]
    pub async fn remove_user_notifications(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let res = notifications::delete_user_notification(
            &session,
            &token.0.token,
            id.0,
        ).await?;

        if res.is_some() {
            Ok(JsonResponse::ok(Value::Null))
        } else {
            Ok(JsonResponse::unauthorized())
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
            None => Ok(JsonResponse::unauthorized()),
            Some(room) =>  Ok(JsonResponse::ok(room)),
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
        match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => Ok(JsonResponse::unauthorized()),
            Some(None) => Ok(JsonResponse::ok(Value::Null)),
            Some(Some(room)) => {
                crate::rooms::set_room_inactive(&session, room).await?;

                Ok(JsonResponse::ok(Value::Null))
            }
        }
    }

    /// Set Current Room Playlist
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
            None => return Ok(JsonResponse::bad_request("No playlist exists with this id.")),
            Some(playlist) => playlist,
        };

        let room = match room_info::get_active_room_for_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(room) =>  room,
        };

        let mut room = match room {
            None => return Ok(JsonResponse::bad_request("User has no active room.")),
            Some(room) => room,
        };

        crate::rooms::set_room_playlist(&session, room.id.clone(), playlist.id).await?;

        room.active_playlist = Some(playlist.id);

        Ok(JsonResponse::Ok(Json(room)))
    }

    /// Set Current Room Now Playing
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
            None => return Ok(JsonResponse::unauthorized()),
            Some(room) =>  room,
        };

        let mut room = match room {
            None => return Ok(JsonResponse::bad_request("User has no active room.")),
            Some(room) => room,
        };

        let active_id = match room.active_playlist.clone() {
            None => return Ok(JsonResponse::bad_request("No playlist selected.")),
            Some(active_id) => active_id,
        };

        let playlist = match get_playlist_by_id(&session, active_id).await? {
            None => return Ok(JsonResponse::bad_request("No playlist exists with this id.")),
            Some(playlist) => playlist,
        };

        if !playlist.items.contains(&entry_id) {
            return Ok(JsonResponse::bad_request("No playlist entry exists for the current playlist."))
        }

        crate::rooms::set_room_currently_playing(
            &session,
            room.id.clone(),
            entry_id.0.clone(),
        ).await?;

        room.playing_now = Some(entry_id.0);

        Ok(JsonResponse::Ok(Json(room)))
    }

    /// Get User Archived Rooms
    ///
    /// Get all archived user rooms.
    #[oai(path = "/users/@me/rooms", method = "get", tag = "ApiTags::User")]
    pub async fn get_user_rooms(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Vec<ArchivedRoom>>> {
        match room_info::get_archived_rooms(&session, &token.0.token).await? {
            None => Ok(JsonResponse::unauthorized()),
            Some(rooms) =>  Ok(JsonResponse::ok(rooms)),
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
            None => Ok(JsonResponse::unauthorized()),
            Some(playlists) =>  Ok(JsonResponse::ok(playlists)),
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
            None => Ok(JsonResponse::unauthorized()),
            Some(entries) =>  Ok(JsonResponse::ok(entries)),
        }
    }

    /// Add User Credits
    ///
    /// Add the user credits associated with a given token.
    #[oai(path = "/users/credit", method = "post", tag = "ApiTags::User")]
    pub async fn add_user_credits(
        &self,
        id: Query<i64>,
        session: Data<&Session>,
        _token: SuperUserBearer,
    ) -> Result<JsonResponse<Value>> {
        let user = user_info::get_user_from_id(&session, id.0).await?;
        if user.is_none() {
            return Ok(JsonResponse::bad_request("This user does not exist."))
        }

        user_info::adjust_user_credits(&session, id.0, 1).await?;

        Ok(JsonResponse::ok(Value::Null))
    }
}

