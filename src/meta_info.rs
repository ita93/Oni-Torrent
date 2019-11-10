use serde_bencode::de;
use std::io::{self, Read};
use serde_bytes::ByteBuf;
use std::fs::File as FsFile;
use std::io::prelude::*;

use crate::error::{Result, Error};

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename="piece length")]
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
    #[serde(rename="root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
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
    #[serde(rename="announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename="creation date")]
    creation_date: Option<i64>,
    #[serde(rename="comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename="created by")]
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
        let mut buffer:Vec<u8>  = Vec::new();
        file.read_to_end(&mut buffer)?;
        let res = de::from_bytes::<TorrentInfo>(&buffer)?;
        Ok(res)
    }

/*    pub fn get_announce_protocol(&self) -> Result<String> {
        self.announce.as_ref().map(|s|{
            String::from(*s.split(':').collect::<Vec<&str>>().first().unwrap())
        }).ok_or(Error::NotSupportProtocol("Cannot read announce url".to_string()))
    }
*/

    pub fn get_announce(&self) -> String {
        match &self.announce {
            Some(s) => {
                s.clone()
            }
            _ => {
                String::new()
            }
        }
    }
}
