mod entries;
mod playlist;

use anyhow::anyhow;
use uuid::Uuid;
use poem::Result;
use poem::web::Data;
use poem_openapi::{Object, OpenApi};
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use serde_json::json;

pub use playlist::*;
pub use entries::*;
use crate::ApiTags;
use crate::db::Session;
use crate::users::user_info;
use crate::utils::{JsonResponse, TokenBearer};


#[derive(Object, Debug)]
pub struct PlaylistCreationPayload {
    #[oai(validator(max_length = 32, min_length = 2))]
    title: String,

    #[oai(validator(max_length = 128, min_length = 2))]
    description: Option<String>,

    #[oai(validator(max_length = 256, pattern=r"https://i\.imgur\.com/[0-9a-z]+\.jpeg|https://i\.imgur\.com/[0-9a-z]+\.png|https://i\.imgur\.com/[0-9a-z]+\.webp"))]
    banner: Option<String>,

    #[oai(default)]
    is_public: bool,

    items: Vec<Uuid>,
}


#[derive(Object, Debug)]
pub struct EntryCreationPayload {
    #[oai(validator(max_length = 32, min_length = 2))]
    title: String,

    #[oai(validator(max_length = 128, min_length = 2))]
    description: Option<String>,

    #[oai(default)]
    is_public: bool,

    #[oai(default)]
    nsfw: bool,

    #[oai(validator(max_length = 256, pattern=r"https://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\(\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+"))]
    ref_link: String,
}

pub struct PlaylistsApi;

#[OpenApi]
impl PlaylistsApi {
    /// Get Playlist
    ///
    /// Get a specific playlist and it's info.
    #[oai(path = "/playlists", method = "get", tag = "ApiTags::Playlists")]
    pub async fn get_playlist(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
    ) -> Result<Json<Option<Playlist>>> {
        Ok(
            playlist::get_playlist_by_id(&session, id.0)
                .await
                .map(|v| Json(v))?
        )
    }

    /// Upvote Playlist
    ///
    /// Upvote a specific playlist returning the newly updated playlist.
    #[oai(path = "/playlists/vote", method = "post", tag = "ApiTags::Playlists")]
    pub async fn upvote_playlist(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Playlist>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::Unauthorized),
            Some(v) => v,
        };

        let mut playlist = match playlist::get_playlist_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "Playlist does not exist."
            })))),
            Some(v) => v,
        };

        if playlist::has_user_voted(&session, user_id, playlist.id.clone()).await? {
            return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "You have already up-voted this playlist in the last 12 hours."
            }))))
        }

        let credits = user_info::get_user_vote_credits(&session, user_id).await?;

        if credits <= 0 {
            return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "You do not have enough credits."
            }))))
        }

        user_info::adjust_user_credits(&session, user_id, -1).await?;
        playlist::upvote_playlist(&session, user_id, playlist.id.clone()).await?;

        playlist.votes += 1;

        Ok(JsonResponse::Ok(Json(playlist)))
    }

    /// Create Playlist
    ///
    /// Creates a playlist from the given payload, returning the fully populated
    /// playlist information (id, etc..).
    #[oai(path = "/playlists", method = "post", tag = "ApiTags::Playlists")]
    pub async fn update_active_room_playlist(
        &self,
        payload: Json<PlaylistCreationPayload>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Playlist>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::Unauthorized),
            Some(v) => v,
        };

        let items = entries::get_entries_with_ids(&session, payload.0.items).await?;

        if items.is_empty() {
            return Ok(JsonResponse::BadRequest(Json(json!({
                "detail": "No valid playlists entries selected."
            }))))
        }

        let is_nsfw = items.iter().any(|v|  v.nsfw);
        let items: Vec<Uuid> = items.into_iter()
            .map(|v| v.id)
            .collect();

        let playlist_id = Uuid::new_v4();
        session.query(
            r#"INSERT INTO playlists (
                id,
                owner_id,
                banner,
                description,
                is_public,
                items,
                nsfw,
                title,
                votes
            ) VALUE (?, ?, ?, ?, ?, ?, ?, 0)"#,
            (
                playlist_id,
                user_id,
                payload.0.banner,
                payload.0.description,
                payload.0.is_public,
                items,
                is_nsfw,
                payload.0.title,
            )
        ).await?;

        let playlist = playlist::get_playlist_by_id(&session, playlist_id)
            .await?
            .ok_or_else(|| anyhow!("expected room in database after creation"))?;

        Ok(JsonResponse::Ok(Json(playlist)))
    }
}