use std::path::PathBuf;

use clap::Parser;
use futures::future::{join_all, try_join_all};
use log::{error, info};
use requests::{BiliMangaClient, BiliMangaError};
use structs::{EpInfo, MangaDetail};
use tokio::task::JoinHandle;

mod requests;
mod structs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    cookie: String,
    #[arg(long, short)]
    url: String,
}

async fn download_episode(
    manga_detail: MangaDetail,
    episode: EpInfo,
    manga_client: BiliMangaClient,
) -> Result<(), BiliMangaError> {
    if episode.is_locked && !episode.is_in_free {
        info!("{}: {} 已锁定", manga_detail.title, episode.title);
        return Ok(());
    }
    info!("正在下载 {}: {}", manga_detail.title, episode.title);
    let base_path = PathBuf::new()
        .join(manga_detail.title.clone())
        .join(episode.ord.to_string());
    if std::fs::exists(base_path.clone()).map_err(BiliMangaError::IoError)? {
        info!("{}: {:?} 已下载，跳过", manga_detail.title, episode.ord);
        return Ok(());
    }
    std::fs::create_dir(base_path.clone()).map_err(BiliMangaError::IoError)?;

    let episode_index = manga_client.get_episode_index(episode.id).await?;
    let decoded_index = manga_client
        .get_decoded_index(
            format!("{}{}", episode_index.host, episode_index.path),
            episode.id,
            manga_detail.id,
        )
        .await?;
    let full_urls = manga_client.get_image_token(decoded_index.pics).await?;
    let tasks = full_urls.into_iter().enumerate().map(|(idx, image_token)| {
        let url = format!("{}?token={}", image_token.url, image_token.token);
        let client = manga_client.clone();
        let base_path = base_path.clone();
        let filename = base_path.join(format!("{}.jpg", idx));
        let task: JoinHandle<Result<(), BiliMangaError>> = tokio::spawn(async move {
            let image = client.download_image(url).await?;
            std::fs::write(filename, image).map_err(BiliMangaError::IoError)?;
            Ok(())
        });
        task
    });
    let results = join_all(tasks).await;
    for r in results {
        r.unwrap()?;
    }
    Ok(())
}

async fn download_manga(args: Args) -> Result<(), BiliMangaError> {
    let manga_client = BiliMangaClient::new(args.cookie)?;
    let user = manga_client.get_user_info().await?;
    info!("你好, {}", user.uname);
    let manga_id = BiliMangaClient::get_manga_id(args.url)?;
    let manga_detail = manga_client.get_manga_detail(manga_id).await?;
    info!("开始下载漫画: {}", manga_detail.title);
    if !std::fs::exists(manga_detail.title.clone()).map_err(BiliMangaError::IoError)? {
        std::fs::create_dir(manga_detail.title.clone()).map_err(BiliMangaError::IoError)?;
    }
    for episode in manga_detail.clone().ep_list {
        download_episode(manga_detail.clone(), episode, manga_client.clone()).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BiliMangaError> {
    pretty_env_logger::init();

    let args = Args::parse();

    match download_manga(args).await {
        Ok(_) => {},
        Err(e) => {
            error!("{:?}", e);
            panic!("can't download manga");
        }
    }
    Ok(())
}
