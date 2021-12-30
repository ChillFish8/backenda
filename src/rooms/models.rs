use poem_openapi::Object;
use uuid::Uuid;
use scylla::FromRow;

use crate::utils::JsSafeBigInt;


#[derive(Object, FromRow)]
pub struct Room {
    pub id: Uuid,
    pub owner_id: JsSafeBigInt,
    pub active: bool,
    pub active_playlist: Option<Uuid>,
    pub banner: Option<String>,
    pub guild_id: Option<JsSafeBigInt>,
    pub invite_only: bool,
    pub is_public: bool,
    pub playing_now: Option<Uuid>,
    pub title: String,
    pub topic: Option<String>,
}
