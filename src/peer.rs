use crate::error::{Error, Result};
use crate::message::{Message, MessageCodec, MessagePlayload};
use crate::signal::Signal;
use crate::downloader::Downloader;

use std::sync::{Arc, Mutex};
use bincode::*;
use bit_vec::BitVec;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::prelude::*;
use tokio::net::{TcpStream, tcp::WriteHalf};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures_util::sink::SinkExt;
use futures::stream::StreamExt;
use priority_queue::PriorityQueue;

/*-----------------------------------Start stuff for this file here ----------------------------------------------*/
const MAXIMUM_REQUEST:i32 = 20;

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
    signal_slot: UnboundedSender<Signal>,
    download_mutex: Arc<Mutex<Downloader>>,
    // FIXME: I think that it is not neccessary to keep a list of requested blocks.
    number_of_requests: i32, 
    is_choke: bool,
}

impl Peer {
    pub fn new(ip_addr: &str, signal_slot: UnboundedSender<Signal>, download_mutex: Arc<Mutex<Downloader>>) -> Peer {
        Self {
            ip_addr: ip_addr.to_string(),
            bit_field: BitVec::new(),
            signal_slot,
            download_mutex,
            number_of_requests: 0,
            is_choke: true,
        }
    }

    pub fn get_bit_field(&self) -> &BitVec {
        &self.bit_field
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

        let mut stream = TcpStream::connect(&self.ip_addr).await?;
        encoded.drain(..7);
        stream.write_all(encoded.as_ref()).await?;

        self.handle_connection(&mut stream).await?;
        Ok(())
    }

    async fn request_more_blocks(&mut self, mut writer: &mut FramedWrite<WriteHalf<'_>, MessageCodec>) {
        //send an interest message.
        writer.send(Message::new(1, Some(2), MessagePlayload::Interest)).await;

        while !self.is_choke && self.number_of_requests < MAXIMUM_REQUEST {
            let mut block_attrs : Option<(u32, u32, u32)> = None;
            //Request fore new block right here.
            //I will keep requesting until the request stack is full.
            //if let Some(block_attrs) = self.download_mutex.lock().unwrap().pick_next_block(&self.bit_field) {
            if let Ok(mut download_instance) = self.download_mutex.lock() {
                if let Some(block_info) = download_instance.pick_next_block(&self.bit_field) {
                    block_attrs = Some((block_info.0, block_info.1, block_info.2));
                } 
            } else {
                return;
            }

            //send request message to partner
            if let Some(attr_val) = block_attrs {
                let msg = Message::new(13, Some(6), MessagePlayload::Request(attr_val.0, attr_val.1, attr_val.2));
                writer.send(msg).await;
                self.number_of_requests += 1;
            }
        }
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
                    self.handle_message(value, &mut writer).await;
                }
            }
            _ => {
                println!("Failed");
            }
        };
        println!("We did get here for: {}", &self.ip_addr);
        Ok(())
    }

    async fn handle_message(&mut self, received_msg: Message, mut writer: &mut FramedWrite<WriteHalf<'_>, MessageCodec>) {
        match received_msg.payload {
            MessagePlayload::BitField(new_bit_field) => {
                self.download_mutex.lock().unwrap().update_priority(new_bit_field.clone());
                println!("Just updated bitfield : {}", self.ip_addr);
                self.bit_field = new_bit_field;
                //try to request a block here
                self.request_more_blocks(&mut writer).await;
            }
            MessagePlayload::Have(pie_idx) => {
                self.bit_field.set(pie_idx as usize, true);
                self.signal_slot.send(Signal::Have(pie_idx as usize));
                //try to request a block here
                self.request_more_blocks(&mut writer).await;
            }
            MessagePlayload::Choke => {
                self.is_choke = true;
            }
            MessagePlayload::UnChoke => {
                self.is_choke = false;
                self.request_more_blocks(&mut writer).await;
            }
            MessagePlayload::Empty => {
                //Mean something, i don't know
            }
            MessagePlayload::Cancel(pie_idx, begin, length) => {
                // Seeder role: Remove a task from job queue and ignore all related reply.
            }
            MessagePlayload::Request(pie_idx, begin, length) => {
                // Seeder role: reply by a data block: MessagePayload::Piece
            }
            MessagePlayload::Piece(pie_idx, begin, data) => {
                // Write to disk, update manager and broadcast a MessagePayload::Have
                // TODO: How to sync Offline field for all Peer?
                //try to request a block here
                self.download_mutex.lock().unwrap().write_block(pie_idx as usize, begin, &data);
                self.number_of_requests -= 1;
                self.request_more_blocks(&mut writer).await;
            }
            MessagePlayload::Port(port) => {
                //We have nothing to do here. I won't support it.
                self.signal_slot.send(Signal::Port(port));
            }
            _ => {
                self.signal_slot.send(Signal::Unknown);
            }
        }
    }
}
