use crate::error::{Error, Result};
use crate::message::{Message, MessagePlayload, MessageCodec};
use bincode::*;
use serde::{Deserialize, Serialize};
use tokio::{
    codec::{FramedRead, FramedWrite},
    net::TcpStream, 
    prelude::*
};
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
}

impl Peer {
    pub fn new(ip_addr: &str) -> Peer {
        println!("New peer: {}", ip_addr);
        Self {
            ip_addr: ip_addr.to_string(),
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
                    println!("{} : id = {:?} {:#x?}", &self.ip_addr, value.id, value.payload);
                    //try to request a piece
                    let msg = Message::new(13, Some(6), MessagePlayload::Request(0, 0, 16384));
                    writer.send(msg).await;
                }
            }
            _ => {
                println!("Failed");
            }
        };
        Ok(())
    }
}
