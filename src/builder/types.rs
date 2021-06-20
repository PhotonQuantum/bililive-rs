use serde::Deserialize;
use url::Url;

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
pub struct RoomQueryResp {
    data: Inner,
}

impl RoomQueryResp {
    pub fn room_id(&self) -> Option<u64> {
        let url = &self.data.url;
        if url.host_str()? != "live.bilibili.com" {
            return None;
        }
        url.path_segments()
            .into_iter()
            .flatten()
            .last()
            .and_then(|id| id.parse().ok())
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize, Hash)]
struct Inner {
    url: Url,
}
