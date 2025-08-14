use std::{process::Stdio, time::SystemTime};

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
    let now = SystemTime::now();
    for _ in 0..1_000_000 {
        framed.send(DataPack::builder().payload(&now).build()).await.unwrap();
    }
}
