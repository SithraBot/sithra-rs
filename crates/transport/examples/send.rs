use std::{
    process::Stdio,
    time::{SystemTime, UNIX_EPOCH},
};

use futures_util::SinkExt;
use sithra_transport::{datapack::DataPack, peer::Peer, util::framed};
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let cmd = Command::new("./recv")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let peer = Peer::from_child(cmd).unwrap();
    let mut framed = framed(peer);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    framed.send(DataPack::builder().payload(&now).build()).await.unwrap();
}
