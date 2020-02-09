use sha1::Sha1;

use crate::meta_info::TorrentInfo;
pub const BLOCKSIZE:u64 = 16384;

#[derive(Clone)]
pub struct Block{
    index: u64,
    length: u64,
    downloaded: bool,
}

impl Block{
    fn new(index: u64, length: u64) -> Self{
        Self{
            index,
            length,
            downloaded: false,
        }
    }
}

pub struct Piece{
   length: u64,
   hash: Sha1,
   blocks: Vec<Block>,
   is_complete: bool,
}

impl Piece {
    pub fn new(length: u64,  hash: Sha1) -> Self {
        let mut blocks = Vec::new();
        let number_of_blocks = (length as f64 / BLOCKSIZE as f64).ceil() as u64;
        for i in 0..number_of_blocks {
            let block_len = if i < (number_of_blocks - 1) {
                BLOCKSIZE
            } else {
                length - (BLOCKSIZE * (number_of_blocks - 1))
            };

            blocks.push(Block::new(i, block_len));
        }

        Self{
            length,
            hash,
            blocks,
            is_complete: false,
        }
    }

    pub fn get_next_block(&self) -> Option<Block> {
        self.blocks.iter().find(|x| x.downloaded == true).map(|x| x.clone())
    }
}

/// This structure will contain lists of pieces
/// 1. A map from piece index to the Piece object. (actually just a vector is enough)
/// 2. A list of piece index that sorted by priority of pieces (the lower first, the higher
///    latter).
/// 3. A list of boundary, this is the first index of the bucket ( a bucket = pieces those have the
///    same priority)
pub struct PieceControler {
    piece_map : Vec<Piece>,
    piece_list : Vec<usize>,
    boundaries : Vec<usize>,
}

impl PieceControler {
    pub fn new(info: &TorrentInfo) -> Self{
        let no_of_pieces = info.get_piece_amount();

        let mut piece_map = Vec::with_capacity(no_of_pieces);
        let piece_list = (0..no_of_pieces).map(|idx| {
            idx as usize
        }).collect::<Vec<usize>>();

        let boundaries = Vec::new();

        Self{
            piece_map,
            piece_list,
            boundaries,
        }
    }
}
