use anyhow::anyhow;
use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};
use poem_openapi::param::Query;
use scylla::IntoTypedRows;
use uuid::Uuid;

use crate::utils::{JsonResponse, SuperUserBearer, TokenBearer};
use crate::ApiTags;
use crate::db::Session;
use crate::rooms::models::Room;
use crate::users::{room_info, user_info};

pub mod models;


#[derive(Object, Debug)]
pub struct RoomCreationPayload {
    #[oai(validator(max_length = 32, min_length = 2))]
    title: String,

    #[oai(validator(max_length = 128, min_length = 2))]
    topic: Option<String>,

    active_playlist: Option<Uuid>,

    #[oai(validator(minimum(value = "0")))]
    guild_id: Option<i64>,

    #[oai(validator(max_length = 256, pattern=r"https://i\.imgur\.com/[0-9a-z]+\.jpeg|https://i\.imgur\.com/[0-9a-z]+\.png|https://i\.imgur\.com/[0-9a-z]+\.webp"))]
    banner: Option<String>,

    #[oai(default)]
    invite_only: bool,

    #[oai(default)]
    is_public: bool,
}



pub struct RoomsApi;

#[OpenApi]
impl RoomsApi {
    /// Get Top Public Rooms
    ///
    /// Gets the top public rooms which are sorted by viewing count.
    #[oai(path = "/rooms/browse/top", method = "get", tag = "ApiTags::Rooms")]
    pub async fn get_top_rooms(
        &self,
        _page: Query<u32>,
        _session: Data<&Session>,
    ) -> Result<JsonResponse<Vec<Room>>> {
        todo!()
    }

    /// Get New Public Rooms
    ///
    /// Gets the newest public rooms which are sorted by creation time.
    #[oai(path = "/rooms/browse/new", method = "get", tag = "ApiTags::Rooms")]
    pub async fn get_new_rooms(
        &self,
        _page: Query<u32>,
        _session: Data<&Session>,
    ) -> Result<JsonResponse<Vec<Room>>> {
        todo!()
    }

    /// Superuser Close Room
    ///
    /// Forcefully closes a room by a superuser.
    #[oai(path = "/rooms/close", method = "delete", tag = "ApiTags::Rooms")]
    pub async fn close_room(
        &self,
        id: Query<Uuid>,
        _token: SuperUserBearer,
        session: Data<&Session>,
    ) -> Result<JsonResponse<Room>> {
        let room = match get_room_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("User already has an active room")),
            Some(r) => r,
        };

        set_room_inactive(&session, room.clone()).await?;

        Ok(JsonResponse::ok(room))
    }

    /// Create Room
    ///
    /// Creates a new room for a given user
    #[oai(path = "/rooms", method = "post", tag = "ApiTags::Rooms")]
    pub async fn create_room(
        &self,
        payload: Json<RoomCreationPayload>,
        token: TokenBearer,
        session: Data<&Session>,
    ) -> Result<JsonResponse<Room>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let active_room = Option::flatten(room_info::get_active_room_for_token(
            &session,
            &token.0.token,
        ).await?);

        if active_room.is_some() {
            return Ok(JsonResponse::bad_request("User already has an active room"))
        }

        let room = create_room_from_payload(&session, user_id, payload.0).await?;
        Ok(JsonResponse::ok(room))
    }

    /// Get Room
    ///
    /// Get a room with a given ID.
    ///
    /// This will return the room info if any of the following conditions are met:
    /// - The user owns the room.
    /// - The room is invite only.
    /// - The room is public.
    /// - The room is private but allows guild members to join and the user
    ///   requesting the room is a member of said guild.
    #[oai(path = "/rooms", method = "get", tag = "ApiTags::Rooms")]
    pub async fn get_room(
        &self,
        id: Query<Uuid>,
        token: TokenBearer,
        session: Data<&Session>,
    ) -> Result<JsonResponse<Room>> {
        let user = match user_info::get_user_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let room = match get_room_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("Room does not exist.")),
            Some(room) => room,
        };

        if room.is_public | room.invite_only | (room.owner_id == user.id) {
            return Ok(JsonResponse::ok(room))
        }

        let guild_id = match room.guild_id {
            None => return Ok(JsonResponse::forbidden()),
            Some(guild_id) => guild_id,
        };

        if user.access_servers.contains_key(&guild_id) {
           Ok(JsonResponse::ok(room))
        } else {
            Ok(JsonResponse::forbidden())
        }
    }
}


async fn create_room_from_payload(
    sess: &Session,
    user_id: i64,
    payload: RoomCreationPayload,
) -> anyhow::Result<Room> {
    let banner = if let Some(url) = payload.banner {
        crate::images::fetch_and_upload(&url).await?
    } else {
        None
    };

    sess.query(
        r#"
        INSERT INTO rooms (
            id,
            owner_id,
            active,
            active_playlist,
            banner,
            guild_id,
            invite_only,
            is_public,
            playing_now,
            title,
            topic
        ) VALUES (uuid(), ?, true, ?, ?, ?, ?, ?, null, ?, ?);
        "#,
        (
            user_id, payload.active_playlist, banner,
            payload.guild_id, payload.invite_only, payload.is_public,
            payload.title, payload.topic,
            )
    ).await?;

    let room = room_info::get_active_room_for_user_id(sess, user_id)
        .await?
        .ok_or_else(|| anyhow!("expected room in database after creation"))?;

    Ok(room)
}


pub async fn get_room_by_id(sess: &Session, id: Uuid) -> anyhow::Result<Option<Room>> {
    let result = sess.query_prepared(
        "SELECT * FROM rooms WHERE id = ?;",
        (id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;


    let room = match rows.into_typed::<Room>().next() {
        None => return Ok(None),
        Some(v) => v?,
    };

    Ok(Some(room))
}

pub async fn set_room_inactive(sess: &Session, room: Room) -> anyhow::Result<()> {
    sess.query_prepared(
        "DELETE FROM rooms WHERE id = ?;",
        (room.id,)
    ).await?;

    sess.query_prepared(
        r#"
        INSERT INTO room_archive (
            id,
            owner_id,
            active_playlist,
            banner,
            guild_id,
            invite_only,
            is_public,
            title,
            topic
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#,
        (
            room.id,
            *room.owner_id,
            room.active_playlist,
            room.banner,
            room.guild_id.map(|v| *v),
            room.invite_only,
            room.is_public,
            room.title,
            room.topic,
            )
    ).await?;

    Ok(())
}

pub async fn set_room_playlist(sess: &Session, id: Uuid, playlist_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "UPDATE rooms SET active_playlist = ? WHERE id = ?;",
        (playlist_id, id)
    ).await?;

    Ok(())
}

pub async fn set_room_currently_playing(sess: &Session, id: Uuid, entry_id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "UPDATE rooms SET playing_now = ? WHERE id = ?;",
        (entry_id, id)
    ).await?;

    Ok(())
}