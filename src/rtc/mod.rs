use std::collections::HashMap;
use poem::Result;
use poem::web::Data;
use poem_openapi::OpenApi;
use poem_openapi::payload::Json;
use poem_openapi::param::Query;
use poem_openapi::Object;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::Value;
use uuid::Uuid;

use crate::ApiTags;
use crate::db::Session;
use crate::utils::TokenBearer;


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
    ) -> Result<Json<Value>> {
        todo!()
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
    ) -> Result<Json<Value>> {
        todo!()
    }
}
