use poem_openapi::Object;
use uuid::Uuid;


#[derive(Object)]
pub struct Room {
    pub id: Uuid,
    pub guild_id: Option<String>,
    pub owner_id: String,
    pub active_playlist: Option<Uuid>,
    pub playing_now: Option<Uuid>,
    pub title: String,
    pub topic: Option<String>,
    pub is_public: bool,
    pub invite_only: bool,
    pub active: bool,
    pub banner: Option<String>
}

pub type RoomInfoNoActive = (
    Uuid, Option<i64>, i64,
    Option<Uuid>, Option<Uuid>, String,
    Option<String>, bool, bool,
    Option<String>
);

pub type RoomInfo = (
    Uuid, Option<i64>, i64,
    Option<Uuid>, Option<Uuid>, String,
    Option<String>, bool, bool,
    Option<String>, bool,
);

impl From<RoomInfoNoActive> for Room {
    fn from(v: RoomInfoNoActive) -> Self {
        Room {
            id: v.0,
            guild_id: v.1.map(|v| v.to_string()),
            owner_id: v.2.to_string(),
            active_playlist: v.3,
            playing_now: v.4,
            title: v.5,
            topic: v.6,
            is_public: v.7,
            invite_only: v.8,
            banner: v.9,
            active: true,
        }
    }
}

impl From<RoomInfo> for Room {
    fn from(v: RoomInfo) -> Self {
        Room {
            id: v.0,
            guild_id: v.1.map(|v| v.to_string()),
            owner_id: v.2.to_string(),
            active_playlist: v.3,
            playing_now: v.4,
            title: v.5,
            topic: v.6,
            is_public: v.7,
            invite_only: v.8,
            banner: v.9,
            active: v.10,
        }
    }
}