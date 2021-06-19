pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) room_id: u64
}

impl Client {
    pub fn room_id(&self) -> u64 {
        self.room_id
    }
}