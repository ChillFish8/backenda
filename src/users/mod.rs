use std::collections::HashMap;
use anyhow::anyhow;
use poem::web::Data;
use poem::Result;
use poem_openapi::auth::Bearer;
use poem_openapi::payload::Json;
use poem_openapi::{OpenApi, Object, SecurityScheme, ApiResponse};
use scylla::{FromRow, IntoTypedRows};

use crate::db::Session;

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
pub struct TokenBearer(Bearer);

#[derive(Object)]
pub struct User {
    id: String,
    access_servers: HashMap<i64, bool>,
    avatar: Option<String>,
    updated_on: i64,
    username: String,
}

#[derive(Object)]
pub struct Guild {
    id: String,
    icon: Option<String>,
    updated_on: i64,
    name: String,
    manager: bool,
}


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


pub struct UsersApi;

#[OpenApi]
impl UsersApi {
    /// Get User
    ///
    /// Get the user data associated with a give token.
    #[oai(path = "/users/@me", method = "get")]
    pub async fn get_user(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<GetUserResp> {
        if let Some(user) = get_user_from_token(&session, &token.0.token).await? {
            Ok(GetUserResp::Ok(Json(user)))
        } else {
            Ok(GetUserResp::Unauthorized)
        }
    }

    /// Get User Guilds
    ///
    /// Get the user guilds data associated with a give token.
    #[oai(path = "/users/@me/guilds", method = "get")]
    pub async fn get_user_guilds(
        &self,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<GetUserGuildsResp> {
        if let Some(guilds) = get_user_guilds_from_token(&session, &token.0.token).await? {
            Ok(GetUserGuildsResp::Ok(Json(guilds)))
        } else {
            Ok(GetUserGuildsResp::Unauthorized)
        }
    }
}


/// Gets a user_id from the given access token if it's valid otherwise return None.
async fn get_user_id_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<i64>> {
    let result = sess.query_prepared(
        "SELECT user_id FROM access_tokens WHERE access_token = ?;",
        (token.to_string(),)
    ).await?;

    let user_id = match result.rows {
        None => None,
        Some(rows) => {
            if let Some(row) = rows.into_typed::<(i64,)>().next() {
                row?.0
            } else {
                None
            }
        }
    };

    Ok(user_id)
}


/// Gets a full user object from the given access token.
async fn get_user_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<User>> {
    let user_id = match get_user_id_from_token(sess, token).await? {
        None => return Ok(None),
        Some(user_id) => user_id,
    };

    let result = sess.query_prepared(
        "SELECT id, access_servers, avatar, updated_on, username FROM users WHERE id = ?;",
        (user_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type UserInfo = (i64, HashMap<i64, bool>, Option<String>, chrono::Duration, String);
    for row in rows.into_typed::<UserInfo>(){
        let row = row?;
        return Ok(Some(User {
            id: row.0.to_string(),
            access_servers: row.1,
            avatar: row.2,
            updated_on: row.3.num_seconds(),
            username: row.4
        }))
    }

    Ok(None)
}


/// Gets all accessible guilds for a given access token.
async fn get_user_guilds_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<Vec<Guild>>> {
    let user = match get_user_from_token(sess, token).await? {
        None => return Ok(None),
        Some(u) => u,
    };

    let mut guilds = vec![];
    for (guild_id, is_manager) in user.access_servers {
        if let Some(guild) = get_guild(sess, guild_id, is_manager).await? {
            guilds.push(guild);
        };
    }

    Ok(Some(guilds))
}


/// Gets a guild based on it's guild id.
async fn get_guild(sess: &Session, guild_id: i64, manager: bool) -> anyhow::Result<Option<Guild>> {
    let result = sess.query_prepared(
        "SELECT id, icon, updated_on, name FROM guilds WHERE id = ?;",
        (guild_id,)
    ).await?;

    let rows = result.rows
        .ok_or_else(|| anyhow!("expected returned rows"))?;

    type GuildInfo = (i64, Option<String>, chrono::Duration, String);
    let res = match rows.into_typed::<GuildInfo>().next() {
        None => None,
        Some(guild) => {
            let guild = guild?;

            Some(Guild {
                id: guild.0.to_string(),
                icon: guild.1,
                updated_on: guild.2.num_seconds(),
                name: guild.3,
                manager,
            })
        }
    };

    Ok(res)
}