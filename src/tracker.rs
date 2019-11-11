use crate::error::{Result, Error};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::prelude::*;
use serde::{Serialize, Deserialize};
use bincode::*;
use rand::prelude::*;
use url::Url;
use bit_vec::BitVec;

/*
 * This file contains all tracker related code.
 */

//static BIND_ADDR: &'static str = "127.0.0.1:11993";
static BIND_ADDR: &'static str = "0.0.0.0:0";

#[derive(Serialize, Debug)]
struct ConnectRequest {
    connection_id: u64,
    action: u32,
    transaction_id: u32,
}

#[derive(Deserialize, Debug)]
struct ConnectResponse {
    action: u32,
    transaction_id: u32,
    connection_id: u64,
}

#[derive(Serialize, Debug)]
struct AnnounceRequest {
    connection_id: u64,
    action: u32,
    transaction_id: u32,
    info_hash: BitVec,
    peer_id: BitVec,
    downloaded: u64,
    left: u64,
    uploaded: u64,
    event: u32,
    ip_address: u32,
    key: u32,
    num_want: u32,
    port: u32,
}

#[derive(Deserialize, Debug)]
struct AnnounceResponse {
    action: u32,
    transaction_id: u32,
    interval: u32,
    leechers: u32,
    seeders: u32,
    peers: Vec<(u32, u16)>, //(ip, port)
}

pub struct Tracker{
    socket: UdpSocket,
    connection_id: Option<u64>, //It is not always here
}

impl Tracker{
    pub async fn from_url(announce_url: String) -> Result<Self> {
        let base_url = Url::parse(&announce_url)?;
        let socket_addrs = base_url.socket_addrs(|| None)?;

        let socket = UdpSocket::bind(&BIND_ADDR.parse::<SocketAddr>()?).await?;
        //FIX ME: It should accept an array of address as std UDP
        let dummy:SocketAddr = "188.241.58.209:6969".parse()?;
        socket.connect(&dummy).await?;
        Ok(Self{
            socket,
            connection_id: None,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        //create a connect message and send it to tracker
        let mut rng = rand::thread_rng(); 
        let request_pkt = ConnectRequest{
            transaction_id: rng.gen(),
            connection_id: 0x41727101980,
            action: 0x0, //connect
        };

        let encoded_pkt: Vec<u8> = bincode::config().big_endian().serialize(&request_pkt)?;
        println!("Sending request");
        self.socket.send(&encoded_pkt).await?;
        let data_size = std::mem::size_of::<ConnectResponse>();
        let mut data = vec![0u8; data_size];
        let len = self.socket.recv(&mut data).await?;
        let decoded_pkt: ConnectResponse = bincode::config().big_endian().deserialize(&data)?;
        println!("Connect response: {:?}", &decoded_pkt);
        //Finish connect, save connection_id for later using
        self.connection_id = Some(decoded_pkt.connection_id);
        Ok(())
    }
}
