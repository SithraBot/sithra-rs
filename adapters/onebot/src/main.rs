use std::{ops::Div, process, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sithra_adapter_onebot::{
    AdapterState, OneBotMessage,
    api::{
        request::ApiCall,
        response::{ApiResponse, ApiResponseKind},
    },
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
use tokio::sync::{Mutex, mpsc, watch};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use triomphe::Arc;
use ulid::Ulid;

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "ws-url")]
    ws_url:                String,
    token:                 Option<String>,
    #[serde(rename = "health-check-interval")]
    #[serde(default = "default_health_check_interval")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    health_check_interval: Duration,
}

const fn default_health_check_interval() -> Duration {
    Duration::from_secs(30)
}

#[tokio::main]
async fn main() {
    // Init plugin
    let (plugin, config) = Plugin::<Config>::new().await.expect("Init adapter onebot failed");

    // config
    let Config {
        ws_url,
        token,
        health_check_interval,
    } = config;

    // create connection manager
    let (conn_manager, ws_rx) = ConnectionManager::new(ws_url, token);
    let ws_tx = conn_manager.ws_tx.clone();

    // init bot
    let bot_id = format!("{}-{}", "onebot", process::id());
    let client = plugin.server.client();

    let state = AdapterState {
        ws_tx: ws_tx.clone(),
    };

    let plugin = plugin.map(|r| {
        r.route_typed(SendMessage::on(send_message))
            .route_typed(SetMute::on(set_mute))
            .layer(BotId::new(bot_id.clone()))
            .with_state(state)
    });

    let (health_tx, health_rx) = watch::channel(true);

    let ws_rx = Arc::new(Mutex::new(ws_rx));
    let connection_set = ConnectionSet {
        ws_rx,
        ws_tx,
        bot_id,
        health_tx,
        health_rx,
    };

    // spawn connection task with auto-reconnect
    let connection_task = tokio::spawn({
        async move {
            conn_manager
                .run_with_reconnect(|ws_stream| {
                    handle_connection(
                        connection_set.clone(),
                        ws_stream,
                        client.sink(),
                        health_check_interval,
                    )
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

#[derive(Clone)]
struct ConnectionSet {
    ws_rx:     Arc<Mutex<mpsc::UnboundedReceiver<WsMessage>>>,
    ws_tx:     mpsc::UnboundedSender<WsMessage>,
    bot_id:    String,
    health_tx: watch::Sender<bool>,
    health_rx: watch::Receiver<bool>,
}

async fn handle_connection(
    ConnectionSet {
        ws_rx,
        ws_tx,
        bot_id,
        health_tx,
        health_rx,
    }: ConnectionSet,
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    sink: ClientSink,
    health_check_interval: Duration,
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

    // spawn receive task
    let recv_task = tokio::spawn(recv_loop(ws_read, bot_id, sink, health_tx));

    let check_req = serde_json::to_string(&ApiCall::new(
        "get_status",
        serde_json::Value::Null,
        Ulid::nil(),
    ))
    .expect(
        "If you see this message, it indicates that an internal error has occurred. Please report \
         the issue.",
    );
    let health_check = tokio::spawn(async move {
        let mut health_rx = health_rx;
        let health_check_interval = health_check_interval;
        let health_check_timeout = health_check_interval.div(2);
        loop {
            tokio::time::sleep(health_check_interval).await;
            let _ = ws_tx.send(WsMessage::text(&check_req));
            let result = tokio::time::timeout(health_check_timeout, health_rx.changed()).await;
            if result.is_err() {
                log::error!("Health check timeout");
                break;
            }
            if !*health_rx.borrow_and_update() {
                log::error!("Health check failed");
                break;
            }
        }
    });

    tokio::select! {
        _ = recv_task => {
            log::error!("Receive task exited unexpectedly.");
        }
        _ = send_task => {
            log::error!("Send task exited unexpectedly.");
        }
        _ = health_check => {
            log::error!("Health check task exited unexpectedly.");
        }
    }
}

async fn recv_loop<S>(
    mut ws_read: S,
    bot_id: String,
    sink: ClientSink,
    health_tx: watch::Sender<bool>,
) where
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

        let message = onebot_adaptation(message, &bot_id, &health_tx);
        if let Some(message) = message {
            if let Err(e) = sink.send(message) {
                log::error!("Failed to send message to sink: {e}");
            }
        }
    }

    log::warn!("WebSocket receive loop ended");
}

fn onebot_adaptation(
    message: WsMessage,
    bot_id: &str,
    health_tx: &watch::Sender<bool>,
) -> Option<DataPack> {
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
        OneBotMessage::Api(ApiResponse {
            data: Some(ApiResponseKind::GetStatus(status)),
            ..
        }) => {
            health_tx.send(status.good).ok();
            None
        }
        OneBotMessage::Api(api) => Some(api.into_rep(bot_id)),
        OneBotMessage::Event(event) => event.into_req(bot_id),
    }
}
