use o_torrent::error::{Result, Error};
use o_torrent::meta_info;
use o_torrent::tracker;

#[tokio::main]
async fn main() -> Result<()>{
    let sample = meta_info::TorrentInfo::from_file("big-buck-bunny.torrent")?;
    let mut tracker = tracker::Tracker::from_url(sample.get_announce()).await?;
    tracker.connect().await?;
    Ok(())
}
