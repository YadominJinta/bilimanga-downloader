use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    MangaDetail(MangaDetail),
    EpInfo(EpInfo),
    BiliUserInfo(BiliUserInfo),
    EpisodeDetail(EpisodeDetail),
    ImageToken(Vec<ImageToken>),
}

impl From<MangaDetail> for ResponseData {
    fn from(value: MangaDetail) -> Self {
        Self::MangaDetail(value)
    }
}

impl From<EpInfo> for ResponseData {
    fn from(value: EpInfo) -> Self {
        Self::EpInfo(value)
    }
}

impl From<BiliUserInfo> for ResponseData {
    fn from(value: BiliUserInfo) -> Self {
        Self::BiliUserInfo(value)
    }
}

impl From<EpisodeDetail> for ResponseData {
    fn from(value: EpisodeDetail) -> Self {
        Self::EpisodeDetail(value)
    }
}

impl From<Vec<ImageToken>> for ResponseData {
    fn from(value: Vec<ImageToken>) -> Self {
        Self::ImageToken(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseResponse<T: Into<ResponseData>> {
    pub code: i64,
    pub message: Option<String>,
    pub msg: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDetail {
    pub id: i64,
    pub title: String,
    pub ep_list: Vec<EpInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpInfo {
    pub id: i64,
    pub title: String,
    pub short_title: String,
    pub is_in_free: bool,
    pub is_locked: bool,
    pub ord: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiliUserInfo {
    pub uname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeDetail {
    pub path: String,
    pub host: String,
    pub images: Vec<ImageDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    pub path: String,
    pub x: i64,
    pub y: i64,
    video_path: String,
    video_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageIndex {
    pub pics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageToken {
    pub url: String,
    pub token: String,
}
