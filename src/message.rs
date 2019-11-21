use tokio::io::BufReader;
use tokio::prelude::*;
use tokio::io::AsyncRead;

enum MessagePlayload<'a>{
    Have(u32), // <piece index>
    BitField(&'a [u8]), // <bitfield> has variant length
    Request(u32, u32, u32), //<index><begin><length>
    Piece(u32, u32, &'a [u8]), //<index><begin><data block>
    Cancel(u32, u32, u32), //<index><begin><length>
    Port(u16), //<port>
    Empty,
}

struct Message<'a> {
    len: u32,
    id: Option<u8>,
    payload: MessagePlayload<'a>,
}

impl <'a> Message<'a>{
    pub fn new(len: u32, id: Option<u8>, payload: MessagePlayload<'a>) -> Self{
        Self {
            len,
            id,
            payload,
        } 
    }
}
