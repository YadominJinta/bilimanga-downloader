use std::{
    collections::HashMap,
    io::{Cursor, Read},
    num::ParseIntError,
};

use const_format::concatcp;
use futures::TryFutureExt;
use log::debug;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder, Url,
};
use zip::{result::ZipError, ZipArchive};

use crate::structs::{
    BaseResponse, BiliUserInfo, EpisodeDetail, ImageIndex, ImageToken, MangaDetail,
};

const BILI_API_BASE_URL: &str = "https://api.bilibili.com";
const BILI_MANGA_BASE_URL: &str = "https://manga.bilibili.com";
const BILI_MANGA_SLB_URL: &str = "https://i0.hdslb.com";
const BILI_MANGA_DETAIL_URL: &str = concatcp!(
    BILI_MANGA_BASE_URL,
    "/twirp/comic.v1.Comic/ComicDetail?device=pc&platform=web"
);
const BILI_API_ME_URL: &str = concatcp!(BILI_API_BASE_URL, "/x/web-interface/nav");
const BILI_MANGA_EPISODE_URL: &str = concatcp!(
    BILI_MANGA_BASE_URL,
    "/twirp/comic.v1.Comic/GetImageIndex?device=pc&platform=web"
);
const BILI_MANGA_IMAGE_TOKEN_URL: &str = concatcp!(
    BILI_MANGA_BASE_URL,
    "/twirp/comic.v1.Comic/ImageToken?device=pc&platform=web"
);
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36";
const JSON_CONTENT: &str = "application/json;charset=UTF-8";

#[derive(Clone, Debug)]
pub struct BiliMangaClient {
    client: Client,
}

#[derive(Debug)]
pub enum BiliMangaError {
    ParseUrlError(url::ParseError),
    ParseIntError(ParseIntError),
    ParseJsonError(serde_json::Error),
    RequestError(reqwest::Error),
    CustomError(String),
    ZipError(ZipError),
    IoError(std::io::Error),
}

impl BiliMangaClient {
    fn load_cookie(cookie_file: String) -> Result<String, BiliMangaError> {
        let mut cookie = String::new();
        let mut file = std::fs::File::open(cookie_file).map_err(BiliMangaError::IoError)?;
        file.read_to_string(&mut cookie)
            .map_err(BiliMangaError::IoError)?;
        Ok(cookie)
    }

    pub fn new(cookie_file: String) -> Result<Self, BiliMangaError> {
        let cookie = Self::load_cookie(cookie_file)?;
        let mut default_headers = HeaderMap::<HeaderValue>::default();
        default_headers.insert("User-Agent", HeaderValue::from_str(USER_AGENT).unwrap());
        default_headers.insert("Content-Type", HeaderValue::from_str(JSON_CONTENT).unwrap());
        default_headers.insert("Cookie", HeaderValue::from_str(&cookie).unwrap());

        let client = ClientBuilder::new()
            .default_headers(default_headers)
            .gzip(true)
            .deflate(true)
            .zstd(true)
            .brotli(true)
            .build()
            .map_err(BiliMangaError::RequestError)?;
        Ok(Self { client })
    }

    pub fn get_manga_id(url: String) -> Result<i64, BiliMangaError> {
        let _ = Url::parse(&url).map_err(BiliMangaError::ParseUrlError)?;
        let splits = url.split("/").collect::<Vec<&str>>();
        let mc_id = splits[splits.len() - 1];
        let mc_id = mc_id.replace("mc", "");
        let id = mc_id
            .parse::<i64>()
            .map_err(BiliMangaError::ParseIntError)?;
        Ok(id)
    }

    pub async fn get_manga_detail(
        self: &Self,
        manga_id: i64,
    ) -> Result<MangaDetail, BiliMangaError> {
        let url: Url = BILI_MANGA_DETAIL_URL.parse().unwrap();
        let response = self
            .client
            .post(url)
            .body(format!("{{\"comic_id\":{}}}", manga_id))
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?;
        let text = response
            .text()
            .await
            .map_err(BiliMangaError::RequestError)?;
        debug!("{}", text);
        let response: BaseResponse<MangaDetail> =
            serde_json::from_str(&text).map_err(BiliMangaError::ParseJsonError)?;
        if response.code != 0 {
            if response.msg.is_some() {
                return Err(BiliMangaError::CustomError(response.msg.unwrap()));
            } else {
                return Err(BiliMangaError::CustomError(response.message.unwrap()));
            }
        }
        Ok(response.data.unwrap())
    }

    pub async fn get_user_info(self: &Self) -> Result<BiliUserInfo, BiliMangaError> {
        let url: Url = BILI_API_ME_URL.parse().unwrap();
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?;
        let headers = response.headers();
        debug!("{:?}", *headers);
        let response_text = response
            .text()
            .await
            .map_err(BiliMangaError::RequestError)?;
        debug!("{}", response_text);
        let response: BaseResponse<BiliUserInfo> =
            serde_json::from_str(&response_text).map_err(BiliMangaError::ParseJsonError)?;
        if response.code != 0 {
            if response.msg.is_some() {
                return Err(BiliMangaError::CustomError(response.msg.unwrap()));
            } else {
                return Err(BiliMangaError::CustomError(response.message.unwrap()));
            }
        }
        Ok(response.data.unwrap())
    }

    pub async fn get_episode_index(
        self: &Self,
        episode_id: i64,
    ) -> Result<EpisodeDetail, BiliMangaError> {
        let url: Url = BILI_MANGA_EPISODE_URL.parse().unwrap();
        let response = self
            .client
            .post(url)
            .body(format!("{{\"ep_id\":{}}}", episode_id))
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?;
        let text = response
            .text()
            .await
            .map_err(BiliMangaError::RequestError)?;
        debug!("{}", text);
        let response: BaseResponse<EpisodeDetail> =
            serde_json::from_str(&text).map_err(BiliMangaError::ParseJsonError)?;
        if response.code != 0 {
            if response.msg.is_some() {
                return Err(BiliMangaError::CustomError(response.msg.unwrap()));
            } else {
                return Err(BiliMangaError::CustomError(response.message.unwrap()));
            }
        }
        Ok(response.data.unwrap())
    }

    pub async fn get_decoded_index(
        self: &Self,
        url: String,
        episode_id: i64,
        manga_id: i64,
    ) -> Result<ImageIndex, BiliMangaError> {
        let url: Url = url.parse().map_err(BiliMangaError::ParseUrlError)?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?
            .bytes()
            .await
            .map_err(BiliMangaError::RequestError)?
            .to_vec();
        let mut response = response[9..].to_vec();
        BiliMangaClient::decode_index(&mut response, episode_id, manga_id);
        let image_index = Self::extract_image_index(response)?;
        Ok(image_index)
    }

    pub async fn get_image_token(
        self: &Self,
        urls: Vec<String>,
    ) -> Result<Vec<ImageToken>, BiliMangaError> {
        let url: Url = BILI_MANGA_IMAGE_TOKEN_URL.parse().unwrap();
        let urls_str = serde_json::ser::to_string(&urls).map_err(BiliMangaError::ParseJsonError)?;
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("urls".into(), urls_str);
        let body = serde_json::ser::to_string(&map).map_err(BiliMangaError::ParseJsonError)?;
        debug!("{}", body);
        let response = self
            .client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?;
        let text = response.text().await.map_err(BiliMangaError::RequestError)?;
        debug!("{}", text);
        let response: BaseResponse<Vec<ImageToken>> = serde_json::from_str(&text).map_err(BiliMangaError::ParseJsonError)?;
        if response.code != 0 {
            if response.msg.is_some() {
                return Err(BiliMangaError::CustomError(response.msg.unwrap()));
            } else {
                return Err(BiliMangaError::CustomError(response.message.unwrap()));
            }
        }
        Ok(response.data.unwrap())
    }

    pub async fn download_image(self: &Self, url: String) -> Result<Vec<u8>, BiliMangaError> {
        let parsed_url: Url = url.parse().map_err(BiliMangaError::ParseUrlError)?;
        let response = self
            .client
            .get(parsed_url)
            .send()
            .await
            .map_err(BiliMangaError::RequestError)?;
        debug!("get image");
        let bytes = response
            .bytes()
            .await
            .map_err(BiliMangaError::RequestError)?;
        Ok(bytes.to_vec())
    }

    fn decode_index(encoded_data: &mut Vec<u8>, episode_id: i64, manga_id: i64) {
        debug!("decode index");
        let key: [u8; 8] = [
            (episode_id & 0xff) as u8,
            (episode_id >> 8 & 0xff) as u8,
            (episode_id >> 16 & 0xff) as u8,
            (episode_id >> 24 & 0xff) as u8,
            (manga_id & 0xff) as u8,
            (manga_id >> 8 & 0xff) as u8,
            (manga_id >> 16 & 0xff) as u8,
            (manga_id >> 24 & 0xff) as u8,
        ];
        for i in 0..encoded_data.len() {
            encoded_data[i] ^= key[i % 8]
        }
        debug!("decode index done");
    }

    fn extract_image_index(data: Vec<u8>) -> Result<ImageIndex, BiliMangaError> {
        let reader = Cursor::new(data);
        let mut zip_file = ZipArchive::new(reader).map_err(BiliMangaError::ZipError)?;
        let file = zip_file.by_index(0).map_err(BiliMangaError::ZipError)?;
        let index: ImageIndex =
            serde_json::from_reader(file).map_err(BiliMangaError::ParseJsonError)?;
        Ok(index)
    }
}
