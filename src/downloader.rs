use crate::meta_info::TorrentInfo;
use crate::piece_control::PieceControler;
use sha1::Sha1;
use bit_vec::BitVec;
use std::collections::HashMap;
pub const BLOCKSIZE:u32 = 16384;
//use sha1::Sha1;

/// At any time the are at most 10 pieces in downloading map.
const MAX_NO_PIECES: i32 = 10;

#[derive(Clone, PartialEq)]
enum BlockState {
    Open, //Can be request
    Requested, //Requesting
    Writing, // Writing data
    Finished, //Download finished
}

struct DownloadingPiece {
    piece_idx: usize,
    blocks: Vec<BlockState>,
    remain_blocks: usize, //keep track here so we don't need to recalculate.
}

pub struct Downloader {
    piece_control: PieceControler, 
    downloading: HashMap<usize, DownloadingPiece>,
    meta_info: TorrentInfo,
}

/*Implementation*/

impl DownloadingPiece {
    fn new(piece_idx: usize, no_blocks: usize) -> Self {
        Self{
            piece_idx, 
            blocks: vec![BlockState::Open; no_blocks],  
            remain_blocks: no_blocks,
        }
    }

    fn set_state(&mut self, block_idx: usize, state: BlockState) {
        self.blocks[block_idx] = state
    }

    fn get_next_free_block(&self) -> Option<usize> {
        self.blocks.iter().position(|block_state| {
           *block_state == BlockState::Open 
        }) 
    }

    fn is_finished(&self) -> bool {
        self.blocks.iter().all(|state| {
            *state == BlockState::Finished
        })
    }
}

impl Downloader {
    pub fn new(torrent_info: &TorrentInfo) -> Self {
        let downloading = HashMap::new();
        let piece_control = PieceControler::new(torrent_info.get_number_of_pieces());

        Self {
            piece_control,
            downloading,
            meta_info: torrent_info.clone(),
        }
    }

    pub fn update_priority(&mut self, bit_field: BitVec) {
        bit_field.iter().enumerate().for_each(|(pie_idx, set)| {
            if set {
                self.piece_control.increase_count(pie_idx);
            }
        });
    }

    pub fn write_block(&mut self, piece_idx: usize, block_offset: u32, data: &[u8]) {
        // Update state to writing first
        if let Some(piece) = self.downloading.get_mut(&piece_idx)  {
            let block_idx = (block_offset / BLOCKSIZE) as usize;
            piece.set_state(block_idx, BlockState::Writing);
            //write block to disk
            //update block state to Finished.
            piece.set_state(block_idx, BlockState::Finished);
            //println!("Downloaded: {} * {}", piece_idx, block_idx);
            if piece.is_finished() {
                self.piece_control.set_piece_complete(piece_idx);
                println!("Piece {} has been finished", piece_idx);
            }
        }
    }

    //FIXME: It should not return a piece that it all blocks are requested
    pub fn pick_next_piece(&mut self, peer_bitfield: &BitVec) -> Option<usize> {
        //Check downding list first
        for idx in self.downloading.keys() {
            if peer_bitfield.get(*idx) == Some(true) {
                if let Some(entry) = self.downloading.get(idx) {
                    //only return if it still has block to request
                    //a lot of if (shit)  
                    if entry.remain_blocks > 0 {
                        return Some(*idx);
                    }
                }
            }
        }
        //There is no valid piece in downloading list, so we get a new one.
        //then add it to downloading list.
        if let Some(piece_idx) = self.piece_control.get_next_piece(&peer_bitfield) {
            self.piece_control.set_piece_picked(piece_idx);

            // Calculate the number of blocks here.
            let number_of_blocks = 
                ((self.meta_info.get_piece_length(piece_idx) as f32) / BLOCKSIZE as f32) as usize;
            let new_piece = DownloadingPiece::new(piece_idx, number_of_blocks);
            self.downloading.insert(piece_idx, new_piece);
            return Some(piece_idx);
        }

        None
    }

    fn get_block_size(&self, piece_idx: usize, block_idx: usize) -> u32 {
        let block_idx = block_idx as u32;
        let piece_length = self.meta_info.get_piece_length(piece_idx);
        if (block_idx + 1) * BLOCKSIZE < piece_length {
            BLOCKSIZE
        } else {
            piece_length - (block_idx * BLOCKSIZE) 
        }
    }

    // What should I return here?
    // return: (piece index, block index, block size)
    pub fn pick_next_block(&mut self, peer_bitfield: &BitVec) -> Option<(u32, u32, u32)> {
        self.pick_next_piece(peer_bitfield).and_then(|piece_idx| {
            if let Some(mut entry) = self.downloading.get_mut(&piece_idx) {
                match entry.get_next_free_block() {
                    Some(block_idx) => {
                        entry.remain_blocks -= 1;
                        entry.set_state(block_idx, BlockState::Requested);
                        Some((piece_idx as u32, block_idx as u32 * BLOCKSIZE, self.get_block_size(piece_idx, block_idx)))
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

