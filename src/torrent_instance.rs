/*From this crate*/
use crate::{
    tracker::Tracker,
    peer::Peer,
    error::{Result, Error},
    meta_info,
};
use std::collections::HashMap;

pub struct TorrentInstance {
    tracker: Tracker,
    peers: HashMap<String, Peer>, 
}

impl TorrentInstance {
    pub async fn new(input: &str) -> Result<Self> {
        let torrent_content = meta_info::TorrentInfo::from_file(input)?;
        let tracker = Tracker::from_metainfo(&torrent_content).await?;
        Ok(Self{
            tracker,
            peers: HashMap::new(),
        })
    }

    pub async fn update_announce(&mut self, num_want: i32, event: u32) -> Result<()> {
        self.tracker.connect().await?;
        self.tracker.announce_request(num_want, event).await?;
        println!("{:?}", self.tracker.get_peers());
        for peer_addr in self.tracker.get_peers() {
            let ip_addr = peer_addr.to_string();
            let peer_id = self.tracker.get_peer_id();
            let hash_info = self.tracker.get_hash_info();
            tokio::spawn(async move {
                let mut peer = Peer::new(&ip_addr);
                peer.send_handshake(peer_id, hash_info).await;
            });
        }
        Ok(())
    }
}
