use rust_embed::Embed;
use sithra_kit::{
    plugin,
    server::extract::payload::Payload,
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

#[derive(Embed)]
#[folder = "lyric"]
struct Asset;

#[tokio::main]
async fn main() {
    let (plugin, _) = plugin!();
    let plugin = plugin.map(|r| r.route_typed(Message::on(random)));
    log::info!("RandLyric started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn random(Payload(msg): Payload<Message<H>>) -> Option<SendMessage> {
    if !matches!(msg.content.as_slice(), [H::Text(c)] if c.eq("随机歌词")) {
        return None;
    }
    let mut lyric_files: Vec<_> = <Asset as Embed>::iter().collect();
    let index = fastrand::usize(..lyric_files.len());
    let lyric_file = lyric_files.swap_remove(index);
    let data = Asset::get(&lyric_file)?.data;
    let lyric = str::from_utf8(&data).ok()?;
    let mut lines: Vec<_> = lyric.lines().filter(|v| !v.trim().is_empty()).collect();
    let index = fastrand::usize(..lines.len());
    let line = lines.swap_remove(index);
    Some(msg!(f "{line}\n\n出自: {lyric_file}"))
}
