use std::process;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sithra_adapter_onebot::{
    AdapterState, OneBotMessage,
    endpoint::{send_message, set_mute},
    util::ConnectionManager,
};
use sithra_kit::{
    layers::BotId,
    plugin::Plugin,
    server::server::ClientSink,
    transport::datapack::DataPack,
    types::{channel::SetMute, message::SendMessage},
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[derive(Clone, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "ws-url")]
    ws_url: String,
    token:  Option<String>,
}

#[tokio::main]
async fn main() {
    // Init plugin
    let (plugin, config) = Plugin::<Config>::new().await.expect("Init adapter onebot failed");

    // config
    let Config { ws_url, token } = config;

    // create connection manager
    let (conn_manager, ws_rx) = ConnectionManager::new(ws_url, token);
    let ws_tx = conn_manager.ws_tx.clone();

    // init bot
    let bot_id = format!("{}-{}", "onebot", process::id());
    let client = plugin.server.client();

    let state = AdapterState { ws_tx };

    let plugin = plugin.map(|r| {
        r.route_typed(SendMessage::on(send_message))
            .route_typed(SetMute::on(set_mute))
            .layer(BotId::new(bot_id.clone()))
            .with_state(state)
    });

    // spawn connection task with auto-reconnect
    let connection_task = tokio::spawn({
        let bot_id = bot_id.clone();

        async move {
            let ws_rx = std::sync::Arc::new(tokio::sync::Mutex::new(ws_rx));

            conn_manager
                .run_with_reconnect(|ws_stream| {
                    handle_connection(ws_stream, ws_rx.clone(), bot_id.clone(), client.sink())
                })
                .await;
        }
    });

    tokio::select! {
        _ = connection_task => {
            log::error!("Connection manager task exited unexpectedly.");
        }
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {
            log::info!("Shutting down OneBot adapter...");
        }
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    ws_rx: std::sync::Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<WsMessage>>>,
    bot_id: String,
    sink: ClientSink,
) {
    let (ws_write, ws_read) = ws_stream.split();

    // spawn send task
    let send_task = tokio::spawn(async move {
        let mut ws_rx = ws_rx.lock().await;
        let mut ws_write = ws_write;

        while let Some(msg) = ws_rx.recv().await {
            if let Err(e) = ws_write.send(msg).await {
                log::error!("Failed to send message to WebSocket: {e}");
                break;
            }
        }
    });

    // run receive loop (blocks until connection drops)
    recv_loop(ws_read, &bot_id, &sink).await;

    // cleanup: abort send task when receive loop exits
    send_task.abort();
    let _ = send_task.await;
}

async fn recv_loop<S>(mut ws_read: S, bot_id: &str, sink: &ClientSink)
where
    S: StreamExt<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(message) = ws_read.next().await {
        let message = match message {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("WebSocket receive error: {e}");
                break;
            }
        };

        let message = onebot_adaptation(message, bot_id);
        if let Some(message) = message {
            if let Err(e) = sink.send(message) {
                log::error!("Failed to send message to sink: {e}");
            }
        }
    }

    log::warn!("WebSocket receive loop ended");
}

fn onebot_adaptation(message: WsMessage, bot_id: &str) -> Option<DataPack> {
    let message = match message.into_text() {
        Ok(message) => message,
        Err(err) => {
            log::error!("Recv message from ws Error: {err}");
            return None;
        }
    };
    if message.is_empty() {
        return None;
    }
    let message = match serde_json::from_str::<OneBotMessage>(&message) {
        Ok(message) => message,
        Err(err) => {
            log::error!("Parse message from ws Error: {err}\traw: {message:?}");
            return None;
        }
    };
    match message {
        OneBotMessage::Api(api) => Some(api.into_rep(bot_id)),
        OneBotMessage::Event(event) => event.into_req(bot_id),
    }
}
