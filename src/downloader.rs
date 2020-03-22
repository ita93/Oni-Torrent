use crate::meta_info::TorrentInfo;
use crate::piece_control::PieceControler;
use crate::error::{Result, Error};
use sha1::Sha1;
use bit_vec::BitVec;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{Read, Write, Seek};
pub const BLOCKSIZE:u32 = 16384;

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
    //should I have a mean to manage file?
    data_file: File,
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
    pub fn new(torrent_info: &TorrentInfo) -> Result<Self> {
        let downloading = HashMap::new();
        let piece_control = PieceControler::new(torrent_info.get_number_of_pieces());
        
        let path = Path::new("downloads").join(torrent_info.get_torrent_name());
        let data_file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
   
        let mut new_instance = Self {
            piece_control,
            downloading,
            meta_info: torrent_info.clone(),
            data_file,
        };
        new_instance.verify()?;
        Ok(new_instance)
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
            //seek
            let piece_offset = self.meta_info.get_piece_length(0) as u64 * piece_idx as u64;
            let block_offset = piece_offset + block_offset as u64;
            self.data_file.seek(std::io::SeekFrom::Start(block_offset as u64));
            //write block to disk
            self.data_file.write_all(data);
            //update block state to Finished.
            piece.set_state(block_idx, BlockState::Finished);
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
        
        //If PC can get here, it means there are some piece stuck in download list or download
        //done.
        None
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

    // Private functions
    fn get_block_size(&self, piece_idx: usize, block_idx: usize) -> u32 {
        let block_idx = block_idx as u32;
        let piece_length = self.meta_info.get_piece_length(piece_idx);
        if (block_idx + 1) * BLOCKSIZE < piece_length {
            BLOCKSIZE
        } else {
            piece_length - (block_idx * BLOCKSIZE) 
        }
    }

    pub fn verify(&mut self) -> Result<()> {

        let mut file = &self.data_file;
        let no_pieces = self.meta_info.get_number_of_pieces();
        for piece_idx in 0..no_pieces {
            let piece_offset = self.meta_info.get_piece_length(0) as u64 * piece_idx as u64;
            let length = self.meta_info.get_piece_length(piece_idx) as usize;
            file.seek(std::io::SeekFrom::Start(piece_offset))?;
            let mut data = Vec::with_capacity(length as usize);
            let read_byte = file.take(length as u64).read_to_end(&mut data)?;
            if read_byte < length {
                //place holder.
                file.write(&vec![0u8; length])?;
            } else {
                //get hash
                let piece_hash = &self.meta_info.get_piece_hash(piece_idx);
                let read_hash = Sha1::from(&data);
                if read_hash.digest().bytes() == *piece_hash {
                    self.piece_control.set_piece_complete(piece_idx);
                    println!("Piece {} hash been download correclty", piece_idx);
                }
            }
        } 
        Ok(()) 
    }
}

