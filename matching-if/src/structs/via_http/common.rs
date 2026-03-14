use crate::types::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum UserIdRequestType {
    Creating,
    Updating,
    Keep(UserId),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserIdResponseType {
    Created(UserId),
    Updated(UserId),
    Keep,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SignalingRequestType {
    OfferAccepting,
    Answering(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SignalingResponseType {
    Offering(String),
    NotRequired,
    AnswerAccepted,
}

impl UserIdRequestType {
    pub fn get_current_user_id(
        &self,
        user_id_response_type: &UserIdResponseType,
    ) -> Result<UserId, String> {
        match self {
            UserIdRequestType::Creating => match user_id_response_type {
                UserIdResponseType::Created(user_id) => Ok(*user_id),
                _ => Err("request type is creating, but response type is not created".into()),
            },
            UserIdRequestType::Updating => match user_id_response_type {
                UserIdResponseType::Updated(user_id) => Ok(*user_id),
                _ => Err("request type is updating, but response type is not updated".into()),
            },
            UserIdRequestType::Keep(user_id) => match user_id_response_type {
                UserIdResponseType::Keep => Ok(*user_id),
                _ => Err("request type is keep, but response type is not keep".into()),
            },
        }
    }
}
