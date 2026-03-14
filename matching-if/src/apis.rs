use hyper::{Body, Method, Request};
use serde::Serialize;

pub trait RequestBuilder {
    fn build_request(&self) -> Result<Request<Body>, String>
    where
        Self: Serialize,
    {
        let json = serde_json::to_string(&self).unwrap();
        let Ok(req) = Request::builder()
            .method(Method::POST)
            .uri(self.get_uri())
            .header("content-type", "application/json")
            .body(Body::from(json))
        else {
            return Err("request builder failed".to_string());
        };
        Ok(req)
    }

    fn get_uri(&self) -> &str;
}
