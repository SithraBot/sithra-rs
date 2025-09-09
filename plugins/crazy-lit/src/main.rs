use std::sync::LazyLock;

use sithra_kit::{
    matchopt, plugin,
    server::extract::payload::Payload,
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

const LIT_LIST: &str = include_str!("../static/01");
static LIT_LEN: LazyLock<usize> = LazyLock::new(|| LIT_LIST.lines().count());

#[tokio::main]
async fn main() {
    let (plugin, _) = plugin!();
    let plugin = plugin.map(|r| r.route_typed(Message::on(crazy)));
    log::info!("CrazyLit plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn crazy(Payload(message): Payload<Message<H>>) -> Option<SendMessage> {
    let name = matchopt!(message.as_slice(), [H::Text(text)] => text.strip_prefix("crazylit"))??;
    let lit = LIT_LIST.lines().nth(fastrand::usize(..*LIT_LEN))?;
    let lit = lit.replace("{target_name}", name.trim()).replace("\\n", "\n");
    Some(msg!(lit))
}
