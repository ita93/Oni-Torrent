use sha1::Sha1;
use std::collections::HashMap;

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

// It should just a structure to store piece position only.
// Because pieces are not always downloading.
pub struct PiecePos{
   peer_count: usize,
   complete: bool,
   piece_list_idx: usize,
}

impl PiecePos {
    pub fn new(piece_idx: usize) -> Self {
        Self{
            peer_count: 0,
            complete: false,
            piece_list_idx: piece_idx,
        }
    }

    /// Increase peer count by one, and also return the new peer count value of this piece
    pub fn increase_count(&mut self) -> usize {
        self.peer_count += 1;
        self.peer_count
    }

    /// Decrease peer count by one, and also return the new peer count value of this piece
    pub fn decrease_count(&mut self) -> usize {
        self.peer_count -= 1;
        self.peer_count
    }

    /// Set position of this piece in piece_list
    pub fn set_list_idx(&mut self, list_idx: usize) {
        self.piece_list_idx = list_idx;
    }

    /// Get current position in piece_list
    pub fn get_list_idx(&self) -> usize {
        self.piece_list_idx
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn get_peer_count(&self) -> usize {
        self.peer_count
    }

    pub fn set_complete(&mut self) {
        self.complete = true;
    }
}

/// This structure will contain lists of pieces
/// 1. A map from piece index to the Piece object. (actually just a vector is enough)
/// 2. A list of piece index that sorted by priority of pieces (the lower first, the higher
///    latter).
/// 3. A list of boundary, this is the last index of the bucket ( a bucket = pieces those have the
///    same priority)
pub struct PieceControler {
    piece_map : Vec<PiecePos>,
    piece_list : Vec<usize>,
    boundaries : HashMap<usize, usize>,
}

impl PieceControler {
    pub fn new(no_of_pieces: usize) -> Self{
        //For test
        let mut piece_map = Vec::with_capacity(no_of_pieces);
        let piece_list = (0..no_of_pieces).map(|idx| {
            piece_map.push(PiecePos::new(idx as usize));
            idx as usize
        }).collect::<Vec<usize>>();

        let mut boundaries = HashMap::new();
        boundaries.insert(0, piece_list.len()-1);
        
        Self{
            piece_map,
            piece_list,
            boundaries,
        }
    }
    
    #[cfg(test)]
    pub fn check_piece_list_invalid(&self) -> bool {
        for i in (0..self.piece_list.len() - 1) {
           if self.piece_map[self.piece_list[i]].peer_count > 
               self.piece_map[self.piece_list[i+1]].peer_count {
                return false;
            }
        }
        true
    }

    /// Increase peer_count of a piece by one
    pub fn increase_count(&mut self, piece_idx: usize) {
        let new_avail = self.piece_map[piece_idx].increase_count();
        let piece_list_idx = self.piece_map[piece_idx].get_list_idx();
        let old_avail = new_avail - 1;
        
        //other_index
        // I'm sure that this one always rather than None otherwise the code is wrong somewhere
        // else
        let old_bound_idx = self.boundaries.get_mut(&old_avail).unwrap(); 

        self.piece_map[piece_idx].set_list_idx(*old_bound_idx);
        self.piece_map[self.piece_list[*old_bound_idx]].set_list_idx(piece_list_idx);
        self.piece_list.swap(piece_list_idx, *old_bound_idx);

        if *old_bound_idx > 0 && self.piece_map[self.piece_list[*old_bound_idx - 1]].peer_count as usize == old_avail {
            *old_bound_idx -= 1;
        } else {
            self.boundaries.remove(&old_avail);
        }

        if let None = self.boundaries.get(&new_avail) {
            self.boundaries.insert(new_avail, self.piece_map[piece_idx].get_list_idx());
        }
    }

    /// Decrease peer_count of piece by one - called when a peer leaves.
    pub fn decrease_count(&mut self, piece_idx: usize) {
       unimplemented!(); 
    }

    pub fn get_no_entries(&self) -> usize {
        self.piece_list.len()
    }
    
    pub fn get_next_piece(&self) -> Option<usize> {
        self.piece_list.iter().position(|&x| {
            self.piece_map[x].get_peer_count() > 0 && !self.piece_map[x].is_complete()
        })
    }

    pub fn set_piece_complete(&mut self, piece_idx: usize) {
        self.piece_map[piece_idx].set_complete();
    }
}

#[cfg(test)]
mod tests {
    use crate::meta_info;
    use super::PieceControler;
    use rand::Rng;
    /*#[test]
    fn check_no_entries() {
        let meta_file = meta_info::TorrentInfo::from_file("big-buck-bunny.torrent").unwrap();
        let piece_control = PieceControler::new(&meta_file);
        assert_eq!(meta_file.get_piece_amount(), piece_control.get_no_entries(), "The piece amount is not match");
    }*/
    
    #[test] 
    fn check_increase10_20() {
        test_gen(10, 20);
    }

    #[test] 
    fn check_increase10_200() {
        test_gen(10, 200);
    }
    
    #[test] 
    fn check_increase100_20() {
        test_gen(100, 20);
    }

    #[test] 
    fn check_increase100_200() {
        test_gen(100, 200);
    }

    fn test_gen(no_pieces: usize, inc_times: i64) {
        let mut piece_control = PieceControler::new(no_pieces);
        let mut rng = rand::thread_rng();
        for i in (0..inc_times) {
            let idx = rng.gen_range(0, no_pieces);
            piece_control.increase_count(idx);
        }
        assert!(piece_control.check_piece_list_invalid());
    }
}