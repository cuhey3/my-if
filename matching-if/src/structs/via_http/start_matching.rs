use crate::structs::via_http::common::{
    SignalingRequestType, SignalingResponseType, UserIdRequestType, UserIdResponseType,
};
use crate::types::{AppId, MatcherId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StartMatchingRequest {
    pub user_id_request_type: UserIdRequestType,
    pub signaling_request_type: SignalingRequestType,
    pub matcher_id: MatcherId,
    pub app_id: AppId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartMatchingResponse {
    pub user_id_response_type: UserIdResponseType,
    pub signaling_response_type: SignalingResponseType,
    pub response_type: StartMatchingResponseType,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StartMatchingResponseType {
    Matched(UserId),
    Waiting,
}
