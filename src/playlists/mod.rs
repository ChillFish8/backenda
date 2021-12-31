mod entries;
mod playlist;

use anyhow::anyhow;
use uuid::Uuid;
use poem::Result;
use poem::web::Data;
use poem_openapi::{Object, OpenApi};
use poem_openapi::param::Query;
use poem_openapi::payload::Json;
use serde_json::Value;

pub use playlist::*;
pub use entries::*;
use crate::ApiTags;
use crate::db::Session;
use crate::users::user_info;
use crate::utils::{JsonResponse, SuperUserBearer, TokenBearer};


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
    ref_link: Option<String>,
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

    /// Get Playlist Entry
    ///
    /// Get a specific entry and it's info.
    #[oai(path = "/entries", method = "get", tag = "ApiTags::Playlists")]
    pub async fn get_playlist_entry(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
    ) -> Result<Json<Option<PlaylistEntry>>> {
        Ok(
            entries::get_entry_by_id(&session, id.0)
                .await
                .map(|v| Json(v))?
        )
    }

    /// Superuser Remove Playlist
    ///
    /// Forcefully removes a playlist by a superuser.
    #[oai(path = "/playlists/override", method = "delete", tag = "ApiTags::Playlists")]
    pub async fn remove_playlist_superuser(
        &self,
        id: Query<Uuid>,
        _token: SuperUserBearer,
        session: Data<&Session>,
    ) -> Result<JsonResponse<Value>> {
        playlist::remove_playlist(&session, id.0).await?;

        Ok(JsonResponse::Ok(Json(Value::Null)))
    }

    /// Superuser Remove Entry
    ///
    /// Forcefully removes a playlist entry by a superuser.
    #[oai(path = "/entries/override", method = "delete", tag = "ApiTags::Playlists")]
    pub async fn remove_entry_superuser(
        &self,
        id: Query<Uuid>,
        _token: SuperUserBearer,
        session: Data<&Session>,
    ) -> Result<JsonResponse<Value>> {
        entries::remove_entry(&session, id.0).await?;

        Ok(JsonResponse::Ok(Json(Value::Null)))
    }

    /// Delete Playlist
    ///
    /// Delete a specific playlist providing the user owns the playlist.
    #[oai(path = "/playlists", method = "delete", tag = "ApiTags::Playlists")]
    pub async fn delete_playlist(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let playlist = match playlist::get_playlist_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("Playlist does not exist.")),
            Some(playlist) => playlist,
        };

        if *playlist.owner_id != user_id {
            return Ok(JsonResponse::forbidden())
        }

        playlist::remove_playlist(&session, playlist.id).await?;

        Ok(JsonResponse::ok(Value::Null))
    }

    /// Delete Playlist Entry
    ///
    /// Delete a specific entry providing the user owns the entry.
    #[oai(path = "/entries", method = "delete", tag = "ApiTags::Playlists")]
    pub async fn delete_playlist_entry(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let entry = match entries::get_entry_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("Playlist does not exist.")),
            Some(entry) => entry,
        };

        if *entry.owner_id != user_id {
            return Ok(JsonResponse::forbidden())
        }

        entries::remove_entry(&session, entry.id).await?;

        Ok(JsonResponse::ok(Value::Null))
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
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let mut playlist = match playlist::get_playlist_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("Playlist does not exist.")),
            Some(v) => v,
        };

        if playlist::has_user_voted(&session, user_id, playlist.id.clone()).await? {
            return Ok(JsonResponse::bad_request(
                "You have already up-voted this playlist in the last 12 hours."
            ))
        }

        let credits = user_info::get_user_vote_credits(&session, user_id).await?;

        if credits <= 0 {
            return Ok(JsonResponse::bad_request("You do not have enough credits."))
        }

        user_info::adjust_user_credits(&session, user_id, -1).await?;
        playlist::upvote_playlist(&session, user_id, playlist.id.clone()).await?;

        playlist.votes += 1;

        Ok(JsonResponse::ok(playlist))
    }

    /// Upvote Playlist Entry
    ///
    /// Upvote a specific playlist entry returning the newly updated entry.
    #[oai(path = "/entries/vote", method = "post", tag = "ApiTags::Playlists")]
    pub async fn upvote_entry(
        &self,
        id: Query<Uuid>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<PlaylistEntry>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let mut entry = match entries::get_entry_by_id(&session, id.0).await? {
            None => return Ok(JsonResponse::bad_request("Entry does not exist.")),
            Some(v) => v,
        };

        if entries::has_user_voted(&session, user_id, entry.id.clone()).await? {
            return Ok(JsonResponse::bad_request(
                "You have already up-voted this entry in the last 12 hours.",
            ))
        }

        let credits = user_info::get_user_vote_credits(&session, user_id).await?;

        if credits <= 0 {
            return Ok(JsonResponse::bad_request("You do not have enough credits."))
        }

        user_info::adjust_user_credits(&session, user_id, -1).await?;
        entries::upvote_playlist(&session, user_id, entry.id.clone()).await?;

        entry.votes += 1;

        Ok(JsonResponse::ok(entry))
    }

    /// Create Playlist
    ///
    /// Creates a playlist from the given payload, returning the fully populated
    /// playlist information (id, etc..).
    ///
    /// Note: This will filter out items to only include valid items. E.g.
    /// Items that are not marked as public when the playlist is public will not be included.
    /// Items that do no exist already as entries will not be included.
    /// If a playlist is *not* public then it will include entries that the user owns.
    #[oai(path = "/playlists", method = "post", tag = "ApiTags::Playlists")]
    pub async fn create_playlist(
        &self,
        payload: Json<PlaylistCreationPayload>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Playlist>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let items = entries::get_entries_with_ids(&session, payload.0.items).await?;
        let is_nsfw = items.iter().any(|v|  v.nsfw);
        let items = filter_valid_entries(user_id, payload.0.is_public, items);

        if items.is_empty() {
            return Ok(JsonResponse::bad_request("No valid playlists entries selected."))
        }

        let playlist_id = Uuid::new_v4();
        let playlist = insert_playlist(
            &session,
                playlist_id,
                user_id,
                payload.0.banner,
                payload.0.description,
                payload.0.is_public,
                items,
                is_nsfw,
                payload.0.title,
            true,
        ).await?.ok_or_else(|| anyhow!("expected item in database after creation"))?;

        Ok(JsonResponse::ok(playlist))
    }

    /// Create Playlist Entry
    ///
    /// Creates a playlist entry from the given payload, returning the fully populated
    /// playlist entry (id, etc..).
    #[oai(path = "/entries", method = "post", tag = "ApiTags::Playlists")]
    pub async fn create_entry(
        &self,
        payload: Json<EntryCreationPayload>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<PlaylistEntry>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let entry_id = Uuid::new_v4();
        let entry = insert_entry(
            &session,
            entry_id,
            user_id,
            payload.0.description,
            payload.0.is_public,
            payload.0.nsfw,
            payload.0.ref_link,
            payload.0.title,
            true,
        ).await?.ok_or_else(|| anyhow!("expected item in database after creation"))?;

        Ok(JsonResponse::ok(entry))
    }

    /// Update Playlist
    ///
    /// Updates a playlist from the given payload, returning the updated, fully populated
    /// playlist information (id, etc..).
    ///
    /// Note: This will filter out items to only include valid items. E.g.
    /// Items that are not marked as public when the playlist is public will not be included.
    /// Items that do no exist already as entries will not be included.
    /// If a playlist is *not* public then it will include entries that the user owns.
    #[oai(path = "/playlists", method = "put", tag = "ApiTags::Playlists")]
    pub async fn update_playlist(
        &self,
        id: Query<Uuid>,
        payload: Json<PlaylistCreationPayload>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Playlist>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let mut playlist = match playlist::get_playlist_by_id(&session, id.0.clone()).await? {
            Some(p) => p,
            None => return Ok(JsonResponse::bad_request("No playlist exists with this id.")),
        };

        if *playlist.owner_id != user_id {
            return Ok(JsonResponse::forbidden())
        }

        let items = entries::get_entries_with_ids(&session, payload.0.items).await?;

        let is_nsfw = items.iter().any(|v|  v.nsfw);
        let items = filter_valid_entries(user_id, payload.0.is_public, items);

        if items.is_empty() {
            return Ok(JsonResponse::bad_request("No valid playlists entries selected."))
        }

        insert_playlist(
            &session,
                id.0,
                user_id,
                payload.0.banner.clone(),
                payload.0.description.clone(),
                payload.0.is_public.clone(),
                items.clone(),
                is_nsfw.clone(),
                payload.0.title.clone(),
            false,
        ).await?;

        playlist.items = items;
        playlist.title = payload.0.title;
        playlist.description = payload.0.description;
        playlist.is_public = payload.0.is_public;
        playlist.nsfw = is_nsfw;
        playlist.banner = payload.0.banner;

        Ok(JsonResponse::ok(playlist))
    }

    /// Update Playlist Entry
    ///
    /// Updates a playlist entry from the given payload, returning the updated, fully populated
    /// playlist entry (id, etc..).
    ///
    /// NOTE: This won't update playlists that use the entry. Instead playlists will be
    /// updated as and when requests (lazily).
    #[oai(path = "/entries", method = "put", tag = "ApiTags::Playlists")]
    pub async fn update_entry(
        &self,
        id: Query<Uuid>,
        payload: Json<EntryCreationPayload>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<PlaylistEntry>> {
        let user_id = match user_info::get_user_id_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(v) => v,
        };

        let mut entry = match entries::get_entry_by_id(&session, id.0.clone()).await? {
            Some(p) => p,
            None => return Ok(JsonResponse::bad_request("No playlist entry exists with this id.")),
        };

        if *entry.owner_id != user_id {
            return Ok(JsonResponse::forbidden())
        }

        insert_entry(
            &session,
            id.0,
            user_id,
            payload.0.description.clone(),
            payload.0.is_public.clone(),
            payload.0.nsfw.clone(),
            payload.0.ref_link.clone(),
            payload.0.title.clone(),
            false,
        ).await?;

        entry.title = payload.0.title;
        entry.ref_link = payload.0.ref_link;
        entry.is_public = payload.0.is_public;
        entry.nsfw = payload.0.nsfw;
        entry.description = payload.0.description;

        Ok(JsonResponse::ok(entry))
    }
}


#[inline]
fn filter_valid_entries(owner_id: i64, is_public: bool, entries: Vec<PlaylistEntry>) -> Vec<Uuid> {
    entries.into_iter()
        .filter(|v| v.is_public | ((*v.owner_id == owner_id) & !is_public))
        .map(|v| v.id)
        .collect()
}


async fn insert_playlist(
    sess: &Session,
    id: Uuid,
    owner_id: i64,
    banner: Option<String>,
    description: Option<String>,
    is_public: bool,
    items: Vec<Uuid>,
    is_nsfw: bool,
    title: String,
    fetch_updated: bool,
) -> anyhow::Result<Option<Playlist>> {
    sess.query(
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
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)"#,
        (
            id,
            owner_id,
            banner,
            description,
            is_public,
            items,
            is_nsfw,
            title,
        )
    ).await?;

    let res = if fetch_updated {
        playlist::get_playlist_by_id(sess, id).await?
    } else {
        None
    };

    Ok(res)
}


async fn insert_entry(
    sess: &Session,
    id: Uuid,
    owner_id: i64,
    description: Option<String>,
    is_public: bool,
    is_nsfw: bool,
    ref_link: Option<String>,
    title: String,
    fetch_updated: bool,
) -> anyhow::Result<Option<PlaylistEntry>> {
    sess.query(
        r#"INSERT INTO playlist_entries (
            id,
            owner_id,
            description,
            is_public,
            nsfw,
            ref_link,
            title,
            votes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, 0)"#,
        (
            id,
            owner_id,
            description,
            is_public,
            is_nsfw,
            ref_link,
            title,
        )
    ).await?;

    let res = if fetch_updated {
        entries::get_entry_by_id(sess, id).await?
    } else {
        None
    };

    Ok(res)
}