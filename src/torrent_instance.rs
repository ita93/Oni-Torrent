/*From this crate*/
use crate::{
    error::{Error, Result},
    meta_info,
    peer::Peer,
    tracker::Tracker,
};
use bit_vec::BitVec;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TorrentInstance {
    tracker: Tracker,
    peers: HashMap<String, Peer>,
    piece_priority: PriorityQueue<usize, usize>, // A pair: piece index - piece priority
}

impl TorrentInstance {
    pub async fn new(input: &str) -> Result<Self> {
        let torrent_content = meta_info::TorrentInfo::from_file(input)?;
        let tracker = Tracker::from_metainfo(&torrent_content).await?;
        Ok(Self {
            tracker,
            peers: HashMap::new(),
            piece_priority: PriorityQueue::with_capacity(torrent_content.get_piece_amount()),
        })
    }

    pub async fn update_announce(&mut self, num_want: i32, event: u32) -> Result<()> {
        self.tracker.connect().await?;
        self.tracker.announce_request(num_want, event).await?;

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
