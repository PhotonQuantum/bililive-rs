use serde::Deserialize;
use url::Url;

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
pub struct Resp<T> {
    data: T,
}

impl Resp<ConfQueryInner> {
    pub fn token(&self) -> &str {
        &self.data.token
    }
    pub fn servers(&self) -> Vec<String> {
        self.data
            .host_server_list
            .iter()
            .map(|server| format!("wss://{}:{}/sub", server.host, server.wss_port))
            .collect()
    }
}

impl Resp<RoomQueryInner> {
    pub fn room_id(&self) -> u64 {
        let url = &self.data.url;
        assert_eq!(url.host_str().unwrap(), "live.bilibili.com");
        url.path_segments()
            .into_iter()
            .flatten()
            .last()
            .unwrap()
            .parse()
            .unwrap()
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
pub struct RoomQueryInner {
    url: Url,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
pub struct ConfQueryInner {
    token: String,
    host_server_list: Vec<WSServer>,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
struct WSServer {
    host: String,
    wss_port: u16,
}
