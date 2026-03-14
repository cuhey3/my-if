use crate::types::{MatcherId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ReceiveSdpOutboundData {
    pub matcher_id: MatcherId,
    pub opponent_id: UserId,
    pub offer: String,
}

#[derive(Deserialize, Serialize)]
pub struct ReceiveSdpReturnData {
    pub answer: String,
}
