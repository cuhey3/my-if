use http::StatusCode;
use serde::{Deserialize, Serialize};

pub trait HttpClientAdapter: Sized {
    fn new() -> Self;
    fn send_json<T: Serialize + Send + Sync>(
        &mut self,
        full_url: &str,
        data: &T,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;

    fn status_is_ok(&self) -> Result<bool, String>;

    fn parse_response_json<'a, T: Deserialize<'a> + Send>(&'a self) -> Result<T, String>;

    fn get_status(&self) -> StatusCode;
}
