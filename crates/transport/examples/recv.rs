#![allow(clippy::cast_precision_loss)]

use std::time::SystemTime;

use futures_util::StreamExt;
use sithra_transport::{peer::Peer, util::framed};

#[tokio::main]
async fn main() {
    let peer = Peer::new();
    let mut framed = framed(peer);
    let mut time_vec = Vec::new();
    for _ in 0..1_000_000 {
        let v = framed.next().await.unwrap();
        let v = v.unwrap().payload::<SystemTime>().unwrap();
        time_vec.push(v);
    }
    let time_vec = time_vec
        .into_iter()
        .map(|t| t.elapsed())
        .map(Result::unwrap)
        .map(|d| d.as_nanos())
        .collect::<Vec<_>>();
    let time = time_vec.iter().sum::<u128>() / (time_vec.len() as u128);

    let max_diff = time_vec.iter().max().unwrap() - time_vec.iter().min().unwrap();
    eprintln!(
        "average latency: {}ms\nmax diff: {}ms",
        time as f64 / 1_000_000f64,
        max_diff as f64 / 1_000_000f64
    );
}
