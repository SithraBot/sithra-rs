use rust_embed::Embed;
use sithra_kit::{
    matchopt, plugin,
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
    let c = matchopt!(msg.content.as_slice(),
        [H::Text(c)] => c.strip_prefix("随机歌词"))??
    .trim();
    let mut lyric_files: Vec<_> = <Asset as Embed>::iter().collect();
    let index = if c.is_empty() {
        fastrand::usize(..lyric_files.len())
    } else {
        let c: usize = c.parse().ok()?;
        if c >= lyric_files.len() {
            return Some(msg!("ID错误"));
        }
        log::info!("随机歌词 ID: {c}");
        c
    };
    let lyric_file = lyric_files.swap_remove(index);
    let data = Asset::get(&lyric_file)?.data;
    let lyric = str::from_utf8(&data).ok()?;
    let mut lines: Vec<_> = lyric.lines().filter(|v| !v.trim().is_empty()).collect();
    let line_index = fastrand::usize(..lines.len());
    let line = lines.swap_remove(line_index);
    Some(msg!(f "{line}\n\n出自: {lyric_file}\n歌曲 ID: {index}"))
}
