use crate::apis::RequestBuilder;
use crate::structs::via_http::common::{
    SdpType, SignalingRequestType, SignalingResponseType, UserIdRequestType,
};
use crate::structs::via_http::send_sdp::{SendSdpRequest, SendSdpResponse};
use crate::structs::via_http::signaling_answer::SignalingAnswerRequest;
use crate::structs::via_http::start_matching::{
    StartMatchingRequest, StartMatchingResponse, StartMatchingResponseType,
};
use crate::structs::via_webrtc::receive_sdp::{ReceiveSdpOutboundData, ReceiveSdpReturnData};
use crate::types::UserId;
use crate::webrtc::peer_connection_wrapper::PeerConnectionWrapper;
use crate::webrtc::web_rtc_wrapper::WebRtcWrapper;
use hyper::body::{HttpBody, to_bytes};
use hyper::{Client, StatusCode};
use tokio::io::{AsyncWriteExt, stdout};

#[derive(Default)]
pub struct MatchingClient {
    user_id: u64,
    opponent_user_id: Option<u64>,
}

impl MatchingClient {
    pub async fn get_peer_connection_wrapper(
        &self,
    ) -> Result<PeerConnectionWrapper, Box<dyn std::error::Error + Send + Sync>> {
        let mut resp = Client::new()
            .request(
                StartMatchingRequest {
                    user_id_request_type: UserIdRequestType::Creating,
                    signaling_request_type: SignalingRequestType::OfferAccepting,
                    matcher_id: 20,
                    app_id: 10,
                }
                .build_request()?,
            )
            .await?;

        if resp.status() == StatusCode::OK {
            let parsed_response: StartMatchingResponse =
                serde_json::from_slice(&to_bytes(resp.into_body()).await?)?;
            let own_user_id = UserIdRequestType::Creating
                .get_current_user_id(&parsed_response.user_id_response_type)
                .map_err(|err| format!("get_current_user_id failed: {}", err))?;

            if let StartMatchingResponseType::Waiting = parsed_response.response_type {
                let SignalingResponseType::Offering(offer) =
                    parsed_response.signaling_response_type
                else {
                    panic!()
                };
                Ok(waiting_logic(own_user_id, offer).await?)
            } else {
                let StartMatchingResponseType::Matched(opponent_id) = parsed_response.response_type
                else {
                    panic!()
                };
                Ok(send_sdp_logic(own_user_id, opponent_id).await?)
            }
        } else {
            while let Some(chunk) = resp.body_mut().data().await {
                stdout().write_all(&chunk?).await?;
            }
            Err("unexpected response from server".into())
        }
    }
}

async fn send_sdp_logic(
    own_user_id: UserId,
    opponent_id: UserId,
) -> Result<PeerConnectionWrapper, Box<dyn std::error::Error + Send + Sync>> {
    let mut wrapper = WebRtcWrapper::default()
        .create_connection_wrapper(own_user_id)
        .await?;
    wrapper.create_offer().await?;
    let offer = wrapper.get_offer()?;
    let mut resp = Client::new()
        .request(
            SendSdpRequest {
                sdp_type: SdpType::Offer(offer.clone()),
                user_id: own_user_id,
                app_id: 10,
                matcher_id: 20,
                opponent_id,
                offer,
            }
            .build_request()?,
        )
        .await?;
    if resp.status() != StatusCode::OK {
        println!("Response error: {}", resp.status());
        while let Some(chunk) = resp.body_mut().data().await {
            stdout().write_all(&chunk?).await?;
        }
        return Err("send sdp request failed".into());
    }
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let parsed_response: SendSdpResponse = serde_json::from_slice(&body_bytes)?;
    println!("finally sdp exchanged {:?}", parsed_response.answer);
    wrapper.set_answer(&parsed_response.answer)?;
    wrapper.load_answer().await?;
    // TODO
    // ここからはアプリケーション間の通信なので専用のIFを使う
    // wrapper
    //     .send_data(&ReceiveSdpOutboundData {
    //         matcher_id: 20,
    //         opponent_id,
    //         offer: format!("hello i am b: {}", own_user_id),
    //     })
    //     .await?;

    Ok(wrapper)
}

async fn waiting_logic(
    user_id: UserId,
    offer: String,
) -> Result<PeerConnectionWrapper, Box<dyn std::error::Error + Send + Sync>> {
    let mut wrapper = WebRtcWrapper::default()
        .create_connection_wrapper(user_id)
        .await?;
    wrapper.create_answer_from_offer(offer).await?;
    let answer = wrapper.get_answer()?;
    let req = SignalingAnswerRequest {
        user_id,
        app_id: 10,
        matcher_id: 20,
        answer,
    }
    .build_request()?;
    let client = Client::new();
    let resp = client.request(req).await?;
    if resp.status() != StatusCode::OK {
        return Err("signaling answer request failed.".into());
    }

    wrapper.ready_to_open_data_channel().await?;
    let Some(message) = wrapper.get_message_receiver().await?.recv().await else {
        return Err("cannot get message from message receiver".into());
    };
    let parsed_response: ReceiveSdpOutboundData = serde_json::from_slice(&message.data)?;
    let mut wrapper2 = WebRtcWrapper::default()
        .create_connection_wrapper(user_id)
        .await?;

    wrapper2
        .create_answer_from_offer(parsed_response.offer)
        .await?;

    let answer = wrapper2.get_answer()?;

    wrapper.send_data(&ReceiveSdpReturnData { answer }).await?;

    wrapper2.ready_to_open_data_channel().await?;
    // let mut wrapper2_message_receiver = wrapper2.get_message_receiver().await?;

    // tokio::select! {
    //     message = wrapper2_message_receiver.recv() => {
    //         println!("message logic");
    //         if let Some(message) = message {
    //             let parsed_response: ReceiveSdpOutboundData = serde_json::from_slice(&message.data)?;
    //             println!("parsed_message offer: {}", parsed_response.offer);
    //         }
    //     }
    // }
    Ok(wrapper2)
}
