use sithra_kit::{
    plugin::Plugin,
    server::extract::payload::Payload,
    types::message::{Message, SendMessage, common::CommonSegment},
};

#[tokio::main]
async fn main() {
    let (plugin, _) = Plugin::new::<()>().await.unwrap();
    let plugin = plugin.map(|r| r.route_typed(Message::on(echo)));
    log::info!("Echo plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn echo(Payload(msg): Payload<Message<CommonSegment>>) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("echo ")?.to_owned();
    log::info!("echo recv: {text}");
    let Message { mut content, .. } = msg;
    {
        let first = content.first_mut()?;
        *first = CommonSegment::text(&text);
    }
    Some(SendMessage::new(content))
}
