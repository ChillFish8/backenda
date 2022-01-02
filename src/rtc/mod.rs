mod events;

use poem::Result;
use poem::web::Data;
use poem_openapi::OpenApi;
use poem_openapi::payload::Json;
use poem_openapi::Object;
use serde_json::Value;
use uuid::Uuid;

use crate::ApiTags;
use crate::db::Session;
use crate::utils::{JsonResponse, TokenBearer};
use crate::users::user_info;
use crate::rooms;
use crate::rtc::events::EventType;


#[derive(Object)]
pub struct CandidateInfo {
    room_id: Uuid,
    candidate: Value,
}


pub struct RtcApi;

#[OpenApi]
impl RtcApi {
    /// Create Offer
    ///
    /// Creates a RTC ICE offer.
    #[oai(path = "/rtc/call/offer", method = "post", tag = "ApiTags::Rtc")]
    pub async fn create_call(
        &self,
        payload: Json<CandidateInfo>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let user = match user_info::get_user_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(u) => u,
        };

        let room = match rooms::get_room_by_id(&session, payload.room_id.clone()).await? {
            None => return Ok(JsonResponse::bad_request("No active room exists with this id.")),
            Some(room) => room,
        };

        if *room.owner_id != *user.id {
            return Ok(JsonResponse::forbidden());
        }

        events::emit_event(
            room.id,
            EventType::CandidateCall,
            payload.0.candidate,
        ).await?;

        Ok(JsonResponse::ok(Value::Null))
    }

    /// Create Answer
    ///
    /// Creates a RTC ICE answer.
    #[oai(path = "/rtc/call/answer", method = "post", tag = "ApiTags::Rtc")]
    pub async fn create_answer(
        &self,
        payload: Json<CandidateInfo>,
        session: Data<&Session>,
        token: TokenBearer,
    ) -> Result<JsonResponse<Value>> {
        let user = match user_info::get_user_from_token(&session, &token.0.token).await? {
            None => return Ok(JsonResponse::unauthorized()),
            Some(u) => u,
        };

        let room = match rooms::get_room_by_id(&session, payload.room_id.clone()).await? {
            None => return Ok(JsonResponse::bad_request("No active room exists with this id.")),
            Some(room) => room,
        };

        let has_guild_access = if let Some(guild_id) = room.guild_id.as_ref() {
          user.access_servers.contains_key(&*guild_id)
        } else {
            false
        };

        if (!room.is_public)                // The room is not public
            & (!room.invite_only)           // The room is not invite only
            & (room.owner_id != user.id)    // They are not the owner of the room
            & (!has_guild_access)           // They don't have access via guilds.
        {
            return Ok(JsonResponse::forbidden())
        }

        events::emit_event(
            room.id,
            EventType::CandidateAnswer,
            payload.0.candidate,
        ).await?;

        Ok(JsonResponse::ok(Value::Null))
    }
}
