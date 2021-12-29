mod discord;

use std::collections::HashMap;
use poem::Result;
use poem::web::Data;
use poem_openapi::OpenApi;
use poem_openapi::payload::Json;
use poem_openapi::param::Query;
use poem_openapi::Object;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::auth::discord::Guild;
use crate::db::Session;

#[derive(Object)]
pub struct ExchangePayload {
    access_token: String
}

pub struct AuthApi;

#[OpenApi]
impl AuthApi {
    /// Exchange code
    ///
    /// Exchanges a Discord code for a generated access token used for
    /// authorization.
    #[oai(path = "/auth/authorize", method = "get")]
    pub async fn exchange_code(
        &self,
        code: Query<String>,
        session: Data<&Session>
    ) -> Result<Json<ExchangePayload>> {
        let access_token = discord::exchange_code(&code.0).await?;

        let user = discord::fetch_user_info(&access_token).await?;
        let guilds = discord::fetch_user_guilds(&access_token).await?;

        update_guilds(&session, &guilds).await?;

        let guilds: HashMap<i64, bool> = guilds.into_iter()
            .map(|v| {
                (v.id, v.is_manager())
            })
            .collect();

        session.query(
            "INSERT INTO users (id, username, avatar, updated_on, access_servers) VALUES (?, ?, ?, toTimeStamp(now()), ?);",
            (user.id, user.username, user.avatar, guilds)
        ).await?;

        let access_token = generate_token(48);
        session.query_prepared(
            "INSERT INTO access_tokens (user_id, access_token) VALUES (?, ?);",
            (user.id, access_token.clone()),
        ).await?;


        Ok(Json(
            ExchangePayload {
                access_token,
            }
        ))
    }

    /// Revoke Token
    ///
    /// Revokes the given access token.
    #[oai(path = "/auth/revoke", method = "post")]
    pub async fn revoke_token(
        &self,
        token: Query<String>,
        session: Data<&Session>,
    ) -> Result<()> {
        session.query_prepared(
            "DELETE FROM access_tokens WHERE access_token = ?;",
            (token.0,)
        ).await?;

        Ok(())
    }
}

async fn update_guilds(sess: &Session, guilds: &[Guild]) -> anyhow::Result<()> {
    for guild in guilds {
        sess.query_prepared(
            "INSERT INTO guilds (id, name, icon, updated_on) VALUES (?, ?, ?, toTimeStamp(now()));",
            (guild.id, guild.name.clone(), guild.icon.to_owned()),
        ).await?;
    }

    Ok(())
}


#[inline]
fn generate_token(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}