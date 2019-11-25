use crate::error::{Error, Result};
use crate::message::{Message, MessagePlayload, MessageCodec};
use bincode::*;
use serde::{Deserialize, Serialize};
use tokio::{
    codec::{FramedRead, FramedWrite},
    net::TcpStream, 
    prelude::*
};
use bit_vec::BitVec;
/* Handshake msg */
#[derive(Debug, Serialize, Deserialize)]
struct HandshakeMsg {
    pstr: String,
    reserved: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20], //peer id
}

pub struct Peer {
    ip_addr: String,
    bit_field: BitVec,
}

impl Peer {
    pub fn new(ip_addr: &str) -> Peer {
        println!("New peer: {}", ip_addr);
        Self {
            ip_addr: ip_addr.to_string(),
            bit_field: BitVec::new(),
        }
    }

    //[u8; 20] implemented Copy trait
    pub async fn send_handshake(&mut self, peer_id: [u8; 20], info_hash: [u8; 20]) -> Result<()> {
        let hsm = HandshakeMsg {
            pstr: String::from("BitTorrent protocol"),
            reserved: [0u8; 8],
            info_hash,
            peer_id,
        };
        let mut encoded: Vec<u8> = bincode::config().big_endian().serialize(&hsm)?;
        //bincode will use 64bit to encode a string length, but we only need 1 byte,
        //so remove first seven bytes.
        encoded.drain(..7);
        let mut stream = TcpStream::connect(&self.ip_addr).await?;
        stream.write_all(encoded.as_ref()).await?;
        //Handle handshake here?
        self.handle_connection(&mut stream).await?;
        println!("I want you to finish here for {}", &self.ip_addr);
        Ok(())
    }

    pub async fn handle_connection(&mut self, mut stream: &mut TcpStream) -> Result<()> {
        let mut data = [0u8; 68];
        match stream.read(&mut data).await {
            Ok(_) => {
                // Suppose that we received correct handshake message first.
                //Trying
                let (r, w) = stream.split();
                let mut reader = FramedRead::new(r, MessageCodec::new());
                let mut writer = FramedWrite::new(w, MessageCodec::new());
                while let Some(Ok(value)) = reader.next().await {
                    // Don't need to care about keep alive message.
                    self.handle_message(value);
                    //try to request a piece
                    /*
                    let msg = Message::new(13, Some(6), MessagePlayload::Request(0, 0, 16384));
                    writer.send(msg).await;
                    */
                }
            }
            _ => {
                println!("Failed");
            }
        };
        Ok(())
    }

    fn handle_message(&mut self, received_msg: Message) {
        match received_msg.payload {
            MessagePlayload::BitField(new_bit_field) => {
                self.bit_field = new_bit_field;
            },
            MessagePlayload::Have(pie_idx) => {
                self.bit_field.set(pie_idx as usize, true);
            },
            MessagePlayload::Empty => {
                //Mean something, i don't know    
                            
            },
            MessagePlayload::Cancel(pie_idx, begin, length) => {
                // Remove a task from job queue and ignore all related reply.
            },
            MessagePlayload::Request(pie_idx, begin, length) => {
                // Seeder role: reply by a data block: MessagePayload::Piece
            },
            MessagePlayload::Piece(pie_idx, begin, data) => {
                // Write to disk, update manager and broadcast a MessagePayload::Have
                // TODO: How to sync Offline field for all Peer?
            },
            MessagePlayload::Port(port) => {
                //We have nothing to do here. I won't support it.
            }
        }
    }
}
