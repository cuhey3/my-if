use crate::structs::via_http::common::SignalingResponseType;
use crate::types::{AppId, MatcherId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SignalingAnswerRequest {
    pub user_id: UserId,
    pub app_id: AppId,
    pub matcher_id: MatcherId,
    pub answer: String,
}

#[derive(Deserialize, Serialize)]
pub struct SignalingAnswerResponse {
    pub signaling_response_type: SignalingResponseType,
}
