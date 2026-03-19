use crate::types::UserId;
use crate::webrtc::matching_client::MatchingClient;
use http_client_if::http_client_adapter::HttpClientAdapter;
use std::marker::PhantomData;
use webrtc_if::peer_connection_adapter::PeerConnectionAdapter;

pub struct MatchingSequence<T: PeerConnectionAdapter, U: HttpClientAdapter> {
    user_id: UserId,
    opponent_user_id: UserId,
    server_domain: String,
    matching_client: Option<MatchingClient<U>>,
    phantom_data: PhantomData<T>,
}

impl<T: PeerConnectionAdapter, U: HttpClientAdapter> MatchingSequence<T, U> {
    pub fn new(server_domain: String) -> Self {
        Self {
            user_id: 0,
            opponent_user_id: 0,
            server_domain,
            matching_client: None,
            phantom_data: PhantomData,
        }
    }

    pub async fn get_peer_connection_wrapper(
        &mut self,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        self.matching_client = Some(MatchingClient::new(self.server_domain.clone()));

        let start_peer_connection_response = self
            .matching_client
            .as_ref()
            .unwrap()
            .start_peer_connection()
            .await?;

        self.user_id = *start_peer_connection_response.get_user_id();

        self.opponent_user_id = *start_peer_connection_response.get_opponent_user_id();

        if let Some(offer) = start_peer_connection_response.get_offer() {
            Ok(self.send_answer_logic(offer).await?)
        } else {
            Ok(self.send_offer_logic().await?)
        }
    }

    async fn send_answer_logic(
        &mut self,
        offer: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let mut wrapper = T::create_connection_wrapper(self.user_id).await?;

        wrapper.create_answer_from_offer(offer).await?;

        let answer = wrapper.get_answer()?;

        self.matching_client
            .as_ref()
            .unwrap()
            .send_answer(self.user_id, self.opponent_user_id, answer)
            .await?;

        wrapper.ready_to_open_data_channel().await?;

        Ok(wrapper)
    }

    async fn send_offer_logic(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let mut wrapper = T::create_connection_wrapper(self.user_id).await?;

        wrapper.create_offer().await?;

        let offer = wrapper.get_offer()?;

        let answer = self
            .matching_client
            .as_ref()
            .unwrap()
            .send_offer(self.user_id, self.opponent_user_id, offer)
            .await?;

        wrapper.set_answer(&answer)?;

        wrapper.load_answer().await?;

        Ok(wrapper)
    }
}
