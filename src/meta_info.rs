use bincode::serialize;
use serde_bencode::de;
use serde_bytes::ByteBuf;
use sha1::Sha1;
use std::fs::File as FsFile;
use std::io::prelude::*;
use std::io::{self, Read};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Clone)]
struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub path: Vec<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TorrentInfo {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

pub fn render_torrent(torrent: &TorrentInfo) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let &Some(ref al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let &Some(ref files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}

impl TorrentInfo {
    pub fn from_file(file_name: &str) -> Result<TorrentInfo> {
        let mut file = FsFile::open(file_name)?;
        //Expecting that the file size is not too large.
        //Normal meta info file should have a small size.
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        let res = de::from_bytes::<TorrentInfo>(&buffer)?;
        Ok(res)
    }

    pub fn get_announce(&self) -> Vec<&str> {
        match &self.announce_list {
            Some(list) => {
                list.iter().flat_map(|array| array.iter()).map(|s| s.as_str()).collect()
            },
            _ => match &self.announce {
                Some(s) => vec![&s],
                _ => Vec::new(),
            }
        }
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        //let encoded_pkt: Vec<u8> = bincode::config().big_endian().serialize(&self.info).unwrap();
        let encoded_pkt = serde_bencode::to_bytes(&self.info).unwrap();
        let hash_entity = Sha1::from(&encoded_pkt);
        hash_entity.digest().bytes()
    }

    pub fn get_number_of_pieces(&self) -> usize {
        // 20 bytes per piece
        self.info.pieces.len() / 20
        /*let total_length = match self.info.length {
            Some(val) => val,
            None => {
                //It means we are having multiple files
                match &self.info.files {
                    Some(files) => files.iter().fold(0, |acc, file| acc + file.length),
                    None => 0,
                }
            }
        };

        (total_length as f64 / self.info.piece_length as f64) as usize
        */
    }

    pub fn get_total_length(&self) -> i64 {
        let total_length = match self.info.length {
            Some(val) => val,
            None => {
                //It means we are having multiple files
                match &self.info.files {
                    Some(files) => files.iter().fold(0, |acc, file| acc + file.length),
                    None => 0,
                }
            }
        };
        total_length
    }
    
    /// Return how many bytes a piece is holding
    pub fn get_piece_length(&self, piece_idx: usize) -> u32 {
        (if (piece_idx + 1) != self.get_number_of_pieces() {
            self.info.piece_length
        } else {
            //last piece
            self.get_total_length() - self.info.piece_length * (self.get_number_of_pieces() as i64 - 1)
        }) as u32
    }

    pub fn get_piece_hash(&self, piece_idx: usize) -> [u8; 20] {
        self.info.pieces.chunks(20).skip(piece_idx).take(1).map(|v| {
            let mut array = [0; 20];
            let bytes = &v[..v.len()];
            array.copy_from_slice(bytes);
            array
        }).next().unwrap_or([0u8; 20])
    }

    pub fn get_torrent_name(&self) -> &str {
        &self.info.name
    }

}
