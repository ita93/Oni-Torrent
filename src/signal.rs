/* 
 * Signal.rs
 * This file use for cross-thread communication (Peer -> Manager)
 */
use bit_vec::BitVec;

#[derive(Debug)]
pub enum Signal {
    Bitfield(&BitVec),
    Have(usize), // Raise when Peer recieve have message. (piece)
    Port(u16),
    Unknown,
}
