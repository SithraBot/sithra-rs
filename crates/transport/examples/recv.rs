use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use sithra_transport::{peer::Peer, util::framed};

#[tokio::main]
async fn main() {
    let peer = Peer::new();
    let mut framed = framed(peer);
    let v = framed.next().await.unwrap();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let v = v.unwrap().payload::<Duration>().unwrap();
    eprintln!("{}", (now - v).as_nanos());
}
