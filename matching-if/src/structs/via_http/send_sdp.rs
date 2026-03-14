use crate::apis::RequestBuilder;
use crate::types::{AppId, MatcherId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SendSdpRequest {
    pub user_id: UserId,
    pub app_id: AppId,
    pub matcher_id: MatcherId,
    pub opponent_id: UserId,
    pub offer: String,
}

impl RequestBuilder for SendSdpRequest {
    fn get_uri(&self) -> &str {
        "http://127.0.0.1:3000/send-sdp"
    }
}

#[derive(Deserialize, Serialize)]
pub struct SendSdpResponse {
    pub answer: String,
}
