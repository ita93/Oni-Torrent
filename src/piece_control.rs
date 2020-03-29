use std::collections::HashMap;
use bit_vec::BitVec;

#[derive(PartialEq)]
enum PieceStatus {
    HAVE,
    NOTYET,
    PICKED, //mean it has been picked and being downling.
}
// It should just a structure to store piece position only.
// Because pieces are not always downloadin/

struct PiecePos{
   peer_count: usize,
   piece_list_idx: usize,
   pub piece_status: PieceStatus,
}

impl PiecePos {
    fn new(piece_idx: usize) -> Self {
        Self{
            peer_count: 0,
            piece_list_idx: piece_idx,
            piece_status: PieceStatus::NOTYET,
        }
    }

    /// Increase peer count by one, and also return the new peer count value of this piece
    fn increase_count(&mut self) -> usize {
        self.peer_count += 1;
        self.peer_count
    }

    /// Decrease peer count by one, and also return the new peer count value of this piece
    fn decrease_count(&mut self) -> usize {
        self.peer_count -= 1;
        self.peer_count
    }

    /// Set position of this piece in piece_list
    fn set_list_idx(&mut self, list_idx: usize) {
        self.piece_list_idx = list_idx;
    }

    /// Get current position in piece_list
    fn get_list_idx(&self) -> usize {
        self.piece_list_idx
    }

    fn get_peer_count(&self) -> usize {
        self.peer_count
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
    finished_piece: usize,
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
        let finished_piece = 0;
        
        Self{
            piece_map,
            piece_list,
            boundaries,
            finished_piece,
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
    
    pub fn get_next_piece(&self, peer_bitfield: &BitVec) -> Option<usize> {
        /*self.piece_list.iter().find_map(|&x| {
            self.piece_map[x].get_peer_count() > 0 && self.piece_map[x].piece_status == PieceStatus::NOTYET
        })*/

        self.piece_list.iter().find_map(|&x| {
            if let Some(check) = peer_bitfield.get(x) {
                if check ==true && self.piece_map[x].piece_status == PieceStatus::NOTYET {
                    return Some(x);
                }
            }
            None
        })
    }
    
    //Post condition: return true if all piece has been finished.
    pub fn set_piece_complete(&mut self, piece_idx: usize) -> bool {
        if self.piece_map[piece_idx].piece_status != PieceStatus::HAVE {
            self.piece_map[piece_idx].piece_status = PieceStatus::HAVE;
            self.finished_piece += 1;
        }

        if self.finished_piece == self.piece_map.len() {
            println!("All piece has been downloaded");
            true
        } else {
            false
        }
    }

    pub fn set_piece_picked(&mut self, piece_idx: usize) {
        self.piece_map[piece_idx].piece_status = PieceStatus::PICKED;
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
        assert_eq!(meta_file.get_number_of_piece(), piece_control.get_no_entries(), "The piece amount is not match");
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
