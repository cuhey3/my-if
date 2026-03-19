use crate::structs::via_http::common::{
    SdpType, SignalingRequestType, SignalingResponseType, UserIdRequestType,
};
use crate::structs::via_http::send_sdp::{SendSdpRequest, SendSdpResponse};
use crate::structs::via_http::start_matching::{
    StartMatchingRequest, StartMatchingResponse, StartMatchingResponseType,
};
use crate::types::UserId;
use http_client_if::http_client_adapter::HttpClientAdapter;
use std::marker::PhantomData;

pub struct MatchingClient<T: HttpClientAdapter> {
    server_domain: String,
    phantom_data: PhantomData<T>,
}

impl<T: HttpClientAdapter> MatchingClient<T> {
    pub fn new(server_domain: String) -> Self {
        MatchingClient {
            server_domain,
            phantom_data: Default::default(),
        }
    }
}

pub struct StartPeerConnectionResponse {
    user_id: UserId,
    opponent_user_id: UserId,
    offer: Option<String>,
}

impl StartPeerConnectionResponse {
    pub fn get_user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn get_opponent_user_id(&self) -> &UserId {
        &self.opponent_user_id
    }

    pub fn get_offer(&self) -> Option<&String> {
        self.offer.as_ref()
    }
}

impl<T: HttpClientAdapter> MatchingClient<T> {
    pub async fn start_peer_connection(
        &self,
    ) -> Result<StartPeerConnectionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = T::new();
        client
            .send_json(
                &format!("{}/start-matching", self.server_domain),
                &StartMatchingRequest {
                    user_id_request_type: UserIdRequestType::Creating,
                    signaling_request_type: SignalingRequestType::OfferAccepting,
                    matcher_id: 20,
                    app_id: 10,
                },
            )
            .await?;

        if client.status_is_ok()? {
            let parsed_response: StartMatchingResponse = client.parse_response_json()?;
            let user_id = UserIdRequestType::Creating
                .get_current_user_id(&parsed_response.user_id_response_type)
                .map_err(|err| format!("get_current_user_id failed: {}", err))?;
            let StartMatchingResponseType::Matched(opponent_user_id) =
                parsed_response.response_type
            else {
                panic!()
            };

            let mut start_peer_connection_response = StartPeerConnectionResponse {
                user_id,
                opponent_user_id,
                offer: None,
            };

            if let SignalingResponseType::Offering(offer) = parsed_response.signaling_response_type
            {
                start_peer_connection_response.offer = Some(offer);
            }
            Ok(start_peer_connection_response)
        } else {
            Err("unexpected response from server".into())
        }
    }

    pub async fn send_answer(
        &self,
        user_id: UserId,
        opponent_id: UserId,
        answer: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = T::new();

        client
            .send_json(
                &format!("{}/send-sdp", self.server_domain),
                &SendSdpRequest {
                    sdp_type: SdpType::Answer(answer),
                    user_id,
                    app_id: 10,
                    matcher_id: 20,
                    opponent_id,
                    offer: "".to_owned(),
                },
            )
            .await?;

        if !client.status_is_ok()? {
            println!("Response error: {}", client.get_status());
            return Err("send sdp request failed".into());
        }

        Ok(())
    }

    pub async fn send_offer(
        &self,
        user_id: UserId,
        opponent_id: UserId,
        offer: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = T::new();

        client
            .send_json(
                &format!("{}/send-sdp", self.server_domain),
                &SendSdpRequest {
                    sdp_type: SdpType::Offer(offer.clone()),
                    user_id,
                    app_id: 10,
                    matcher_id: 20,
                    opponent_id,
                    offer,
                },
            )
            .await?;

        if !client.status_is_ok()? {
            println!("Response error: {}", client.get_status());
            return Err("send sdp request failed".into());
        }

        let parsed_response: SendSdpResponse = client.parse_response_json()?;

        println!("finally sdp exchanged {:?}", parsed_response.answer);

        Ok(parsed_response.answer)
    }
}
