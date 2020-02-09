/*From this crate*/
use crate::downloader::Downloader;
use crate::{
    error::{Error, Result},
    meta_info,
    peer::Peer,
    signal::Signal,
    tracker::Tracker,
};
use bit_vec::BitVec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct TorrentInstance {
    tracker: Tracker,
    peers: HashMap<String, Peer>,
    downloader: Arc<Mutex<Downloader>>,
}

impl TorrentInstance {
    pub async fn new(input: &str) -> Result<Self> {
        let torrent_content = meta_info::TorrentInfo::from_file(input)?;
        let tracker = Tracker::from_metainfo(&torrent_content).await?;
        let downloader = Arc::new(Mutex::new(Downloader::new(&torrent_content)));
        Ok(Self {
            tracker,
            peers: HashMap::new(),
            downloader,
        })
    }

    pub async fn update_announce(&mut self, num_want: i32, event: u32) -> Result<()> {
        self.tracker.connect().await?;
        self.tracker.announce_request(num_want, event).await?;

        let (tx, mut rx) = mpsc::unbounded_channel();

        for peer_addr in self.tracker.get_peers() {
            let ip_addr = peer_addr.to_string();
            let peer_id = self.tracker.get_peer_id();
            let hash_info = self.tracker.get_hash_info();

            let peer_tx = tx.clone();
            let cloned_downloader = self.downloader.clone();

            tokio::spawn(async move {
                let mut peer = Peer::new(&ip_addr, peer_tx, cloned_downloader);
                peer.send_handshake(peer_id, hash_info).await;
            });
        }

        while let Some(msg) = rx.recv().await {
            println!("{:?}", msg);
        }

        Ok(())
    }
}
