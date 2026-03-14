use crate::apis::RequestBuilder;
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

impl RequestBuilder for SignalingAnswerRequest {
    fn get_uri(&self) -> &str {
        "http://127.0.0.1:3000/signaling-answer"
    }
}
