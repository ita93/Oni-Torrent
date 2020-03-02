use bit_vec::BitVec;
use bytes::{BufMut, BytesMut};
use tokio::io::AsyncRead;
use tokio::io::{self, BufReader};
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder};

use crate::error::{Error, Result};

const PIECE_MSG_PREFIX_LENGTH: usize = 8;

#[derive(Debug)]
pub enum MessagePlayload {
    Have(u32),                // <piece index>
    BitField(BitVec),         // <bitfield> has variant length
    Request(u32, u32, u32),   //<index><begin><length>
    Piece(u32, u32, Vec<u8>), //<index><begin><data block>
    Cancel(u32, u32, u32),    //<index><begin><length>
    Port(u16),                //<port>
    Choke,
    UnChoke,
    Interest,
    NotInterest,
    Empty,
}

pub struct Message {
    len: usize,
    pub id: Option<u8>,
    pub payload: MessagePlayload,
}

impl Message {
    pub fn new(len: usize, id: Option<u8>, payload: MessagePlayload) -> Self {
        Self { len, id, payload }
    }

    pub fn parse(reader: impl AsyncRead + std::marker::Unpin) -> Result<()> {
        let mut reader_buf = BufReader::new(reader);
        let mut temp = [0; 4];
        //read msg length (first 4 bytes)
        reader_buf.read_exact(&mut temp);
        let length = u32::from_be_bytes(temp);
        //read id
        let mut temp = [0; 1];
        reader_buf.read_exact(&mut temp);
        //Read payload

        Ok(())
    }
}

/// This Codec will be used to encode/decode Message
pub struct MessageCodec {
    id: Option<u8>,
    len: usize,
}

impl MessageCodec {
    pub fn new() -> Self {
        Self { id: None, len: 0 }
    }
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        buf.put_u32(msg.len as u32);
        if let Some(val) = msg.id {
            buf.put_u8(val);

            //Suppose that id and payload are matched.
            match msg.payload {
                MessagePlayload::BitField(bit_field) => {
                    buf.put(&bit_field.to_bytes()[..]);
                }
                MessagePlayload::Have(pie_index) => {
                    buf.put_u32(pie_index);
                }
                MessagePlayload::Request(index, begin, length) => {
                    buf.put_u32(index);
                    buf.put_u32(begin);
                    buf.put_u32(length);
                }
                MessagePlayload::Piece(index, begin, data) => {
                    buf.put_u32(index);
                    buf.put_u32(begin);
                    buf.put(&data[..]);
                }
                MessagePlayload::Cancel(index, begin, length) => {
                    buf.put_u32(index);
                    buf.put_u32(begin);
                    buf.put_u32(length);
                }
                MessagePlayload::Port(port) => {
                    buf.put_u16(port);
                }
                _ => { /*Do nothing*/ } //Choke, Unchoke, Interest and Non-interest don't have payload.
            }
        }

        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Message>> {
        let (id, len) = match self.id {
            Some(val) => (val, self.len),
            None => {
                let raw_len = buf.len();
                if raw_len < 4 {
                    return Ok(None);
                }
                let mut msg_len: [u8; 4] = std::default::Default::default();
                msg_len.copy_from_slice(buf[..4].as_ref());
                let len = u32::from_be_bytes(msg_len) as usize;
                match len {
                    0 => {
                        buf.split_to(4);
                        return Ok(Some(Message::new(0, None, MessagePlayload::Empty)));
                    },
                    _ if buf.len() < 5 => {
                        return Ok(None);
                    },
                    _ => {
                        buf.split_to(4);
                        let id = buf.split_to(1);
                        (id[0], len)
                    }
                }
            }
        };

        
        //Only process if we had received data fully.
        if len - 1 > buf.len() {
            self.id = Some(id);
            self.len = len;
            return Ok(None);
        } else {
            self.id = None;
        }

        match id {
            raw_id @ 0 => {
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Choke,
                )))
            }
            raw_id @ 1 => {
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::UnChoke,
                )))
            }
            raw_id @ 2..=3 => {
                //No payload team
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Empty,
                )))
            }
            raw_id @ 4 => {
                // Have
                let mut temp: [u8; 4] = std::default::Default::default();
                temp.copy_from_slice(&buf.split_to(len - 1));
                let pie_index = u32::from_be_bytes(temp);
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Have(pie_index),
                )))
            }
            raw_id @ 5 => {
                // Bit field
                let bit_field = BitVec::from_bytes(&buf.split_to(len - 1));
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::BitField(bit_field),
                )))
            }
            raw_id @ 6 => {
                // Request for a block of piece
                let mut temp: [u8; 4] = std::default::Default::default();
                temp.copy_from_slice(&buf.split_to(4));
                let index = u32::from_be_bytes(temp);

                temp.copy_from_slice(&buf.split_to(4));
                let begin = u32::from_be_bytes(temp);

                temp.copy_from_slice(&buf.split_to(4));
                let length = u32::from_be_bytes(temp);

                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Request(index, begin, length),
                )))
            }
            raw_id @ 7 => {
                // Data block
                // Only read when we received enough message.
                let mut temp: [u8; 4] = std::default::Default::default();
                temp.copy_from_slice(&buf.split_to(4));
                let index = u32::from_be_bytes(temp);

                temp.copy_from_slice(&buf.split_to(4));
                let begin = u32::from_be_bytes(temp);

                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Piece(index, begin, buf.split_to(len - 9).to_vec()),
                )))
            }

            raw_id @ 8 => {
                // Cancle request - semilar to Request message
                let mut temp: [u8; 4] = std::default::Default::default();
                temp.copy_from_slice(&buf.split_to(4));
                let index = u32::from_be_bytes(temp);

                temp.copy_from_slice(&buf.split_to(4));
                let begin = u32::from_be_bytes(temp);

                temp.copy_from_slice(&buf.split_to(4));
                let length = u32::from_be_bytes(temp);

                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Cancel(index, begin, length),
                )))
            }

            raw_id @ 9 => {
                // Listening port
                let mut temp: [u8; 2] = std::default::Default::default();
                temp.copy_from_slice(&buf.split_to(2));
                let port = u16::from_be_bytes(temp);
                Ok(Some(Message::new(
                    len,
                    Some(raw_id),
                    MessagePlayload::Port(port),
                )))
            }
            _ => Ok(None),
        }
    }
}
