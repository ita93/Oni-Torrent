use o_torrent::error::{Error, Result};
use o_torrent::meta_info;
use o_torrent::peer;
use o_torrent::torrent_instance;
use o_torrent::tracker;

#[tokio::main]
async fn main() -> Result<()> {
    /*let sample = meta_info::TorrentInfo::from_file("big-buck-bunny.torrent")?;
    let mut tracker = tracker::Tracker::from_metainfo(&sample).await?;
    tracker.connect().await?;
    tracker.annouce_request(-1, 0).await?;
    */
    let mut instance = torrent_instance::TorrentInstance::new("test.torrent").await?;
    instance.update_announce(-1i32, 0u32).await?;
    Ok(())
}
