use crate::meta_info::TorrentInfo;
use priority_queue::PriorityQueue;
use bit_vec::BitVec;
use std::collections::HashMap;
//use sha1::Sha1;
use crate::piece::Piece;

//pub const BLOCKSIZE:u64 = 16384;
/// At any time the are at most 10 pieces in downloading map.
const MAX_NO_PIECES: usize = 10;

pub struct Downloader {
    piece_priorities: PriorityQueue<usize, usize>,
    downloading: HashMap<usize, Piece>,
}

impl Downloader {
    pub fn new(torrent_info: &TorrentInfo) -> Self {
        let mut piece_priorities = PriorityQueue::with_capacity(torrent_info.get_piece_amount());
        (0..piece_priorities.capacity()).for_each(|x| {
            piece_priorities.push(x, 0);
        });

        Self { 
            piece_priorities,
            downloading: HashMap::new(),
        }
    }

    pub fn update_priority(&mut self, bit_field: BitVec) {
        bit_field.iter().enumerate().for_each(|(pie_idx, set)| {
            let mut priority = *self.piece_priorities.get_priority(&pie_idx).unwrap_or(&0);
            priority += set as usize;
            self.piece_priorities.change_priority(&pie_idx, priority);
        });

        println!("Just updated bitfield ");
    }


    pub fn write_block(&mut self, piece_idx: usize, block_idx: usize){
        //write block to disk
        //update block field
    }
}

