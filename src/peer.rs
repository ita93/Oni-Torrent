use crate::error::{Error, Result};
use bincode::*;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, prelude::*};

/* Handshake msg */
#[derive(Debug, Serialize)]
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

        Ok(())
    }
}
