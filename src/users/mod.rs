use std::collections::HashMap;
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

#[derive(Object, FromRow)]
pub struct User {
    id: String,
    access_servers: HashMap<i64, bool>,
    avatar: Option<String>,
    updated_on: i64,
    username: String,
}

#[derive(ApiResponse)]
pub enum GetUserResp {
    #[oai(status = 200)]
    Ok(Json<User>),

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
}


async fn get_user_from_token(sess: &Session, token: &str) -> anyhow::Result<Option<User>> {
    let result = sess.query_prepared(
        "SELECT user_id FROM access_tokens WHERE access_token = ?;",
        (token.to_string(),)
    ).await?;

    let user_id = match result.rows {
        None => return Ok(None),
        Some(rows) => {
            if let Some(row) = rows.into_typed::<(i64,)>().next() {
                row?.0
            } else {
                return Ok(None)
            }
        }
    };
    let result = sess.query_prepared(
        "SELECT id, access_servers, avatar, updated_on, username FROM users WHERE id = ?;",
        (user_id,)
    ).await?;

    if let Some(rows) = result.rows {
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
    }

    return Ok(None)
}