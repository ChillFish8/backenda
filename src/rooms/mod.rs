use anyhow::anyhow;
use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};
use poem_openapi::param::Query;
use scylla::IntoTypedRows;
use serde_json::json;
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

    #[oai(validator(max_length = 48, min_length = 2))]
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
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "User already has an active room"
            })))),
            Some(r) => r,
        };

        set_room_inactive(&session, room.id.clone()).await?;

        Ok(JsonResponse::Ok(Json(room)))
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
            None => return Ok(JsonResponse::Unauthorized),
            Some(v) => v,
        };

        let active_room = Option::flatten(room_info::get_active_room_for_token(
            &session,
            &token.0.token,
        ).await?);

        if active_room.is_some() {
            return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "User already has an active room"
            }))))
        }

        let room = create_room_from_payload(&session, user_id, payload.0).await?;
        Ok(JsonResponse::Ok(Json(room)))
    }
}


async fn create_room_from_payload(
    sess: &Session,
    user_id: i64,
    payload: RoomCreationPayload,
) -> anyhow::Result<Room> {
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
            user_id, payload.active_playlist, payload.banner,
            payload.guild_id, payload.invite_only, payload.is_public,
            payload.title, payload.topic,
            )
    ).await?;

    let room = room_info::get_active_room_for_user_id(sess, user_id)
        .await?
        .ok_or_else(|| anyhow!("expected room in database after creation"))?;

    Ok(room)
}


async fn get_room_by_id(sess: &Session, id: Uuid) -> anyhow::Result<Option<Room>> {
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

pub async fn set_room_inactive(sess: &Session, id: Uuid) -> anyhow::Result<()> {
    sess.query_prepared(
        "UPDATE rooms SET active = false WHERE id = ?;",
        (id,)
    ).await?;

    Ok(())
}

pub async fn set_room_playlist(sess: &Session, id: Uuid, playlist_id: Uuid, is_nsfw: bool) -> anyhow::Result<()> {
    sess.query_prepared(
        "UPDATE rooms SET active_playlist = ?, nsfw = ? WHERE id = ?;",
        (playlist_id, is_nsfw, id)
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