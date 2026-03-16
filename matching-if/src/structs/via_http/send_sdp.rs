use crate::apis::RequestBuilder;
use crate::structs::via_http::common::{SdpType, SignalingResponseType};
use crate::types::{AppId, MatcherId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SendSdpRequest {
    pub sdp_type: SdpType,
    pub user_id: UserId,
    pub app_id: AppId,
    pub matcher_id: MatcherId,
    pub opponent_id: UserId,
    pub offer: String,
}

impl RequestBuilder for SendSdpRequest {
    fn get_uri(&self) -> &str {
        "http://127.0.0.1:3000/web_rtc/send-sdp"
    }
}

#[derive(Deserialize, Serialize)]
pub struct SendSdpResponse {
    pub answer: String,
    pub opponent_user_id: UserId,
    pub signaling_response_type: SignalingResponseType,
}
