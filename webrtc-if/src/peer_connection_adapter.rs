use serde::Serialize;

#[derive(Debug)]
pub enum PeerType {
    Offerer,
    Answerer,
}

pub trait PeerConnectionAdapter: Sized {
    fn create_offer(&mut self) -> impl Future<Output = Result<String, String>>;

    fn get_offer(&self) -> Result<String, String>;

    fn set_answer(&mut self, answer: &str) -> Result<(), String>;

    fn load_answer(&mut self) -> impl Future<Output = Result<(), String>>;

    fn ready_to_open_data_channel(&mut self) -> impl Future<Output = Result<(), String>>;

    fn create_answer_from_offer(&mut self, offer: &str)
    -> impl Future<Output = Result<(), String>>;

    fn get_answer(&self) -> Result<String, String>;

    fn is_offerer(&self) -> bool;

    fn send_json(&self, json: &str) -> impl Future<Output = Result<usize, String>>;

    fn wait_message_json(&self) -> impl Future<Output = Result<String, String>>;

    fn create_connection_wrapper(user_id: u64) -> impl Future<Output = Result<Self, String>>;

    fn get_user_id(&self) -> &u64;

    fn send_data<T: Serialize + Sync>(
        &self,
        data: &T,
    ) -> impl Future<Output = Result<usize, String>>;

    fn close(&mut self) -> impl Future<Output = Result<(), String>>;
}
