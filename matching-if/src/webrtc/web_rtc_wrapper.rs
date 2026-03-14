use crate::webrtc::peer_connection_wrapper::PeerConnectionWrapper;
use webrtc::api::{API, APIBuilder};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;

pub struct WebRtcWrapper {
    rtc_configuration: RTCConfiguration,
    api: API,
}

impl Default for WebRtcWrapper {
    fn default() -> Self {
        let registry = Registry::new();
        let api = APIBuilder::new()
            .with_interceptor_registry(registry)
            .build();

        let rtc_configuration = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };
        Self {
            rtc_configuration,
            api,
        }
    }
}

impl WebRtcWrapper {
    pub async fn create_connection_wrapper(
        &self,
        user_id: u64,
    ) -> Result<PeerConnectionWrapper, String> {
        let Ok(peer_connection) = self
            .api
            .new_peer_connection(self.rtc_configuration.clone())
            .await
        else {
            return Err("create peer connection failed.".into());
        };

        Ok(PeerConnectionWrapper::new(user_id, peer_connection))
    }
}
