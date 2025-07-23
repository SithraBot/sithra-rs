use axum::Router;
use schemars::JsonSchema;
use serde::Deserialize;
use sithra_kit::{
    plugin::Plugin,
    server::server::Client,
    transport::channel::{Channel, ChannelType},
    types::initialize::Initialize,
};
use tokio::net::TcpListener;

mod event;
mod webhook;

use webhook::webhook;

#[derive(Deserialize, JsonSchema)]
struct Config {
    /// # webhook 端口
    port:     u16,
    /// # webhook 地址
    host:     String,
    /// # webhook 密钥
    secret:   String,
    /// # 广播频道
    channels: Vec<ChannelConfig>,
}

#[derive(Deserialize, JsonSchema)]
struct ChannelConfig {
    /// # 机器人ID
    #[serde(rename = "bot-id")]
    bot_id: String,
    /// # 频道类型
    #[serde(flatten)]
    kind:   ChannelKind,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum ChannelKind {
    /// # 群组频道
    Group(
        /// # 群组ID
        String,
    ),
    /// # 私人频道
    Private(
        /// # 用户ID
        String,
    ),
}

impl From<ChannelConfig> for (Channel, String) {
    fn from(value: ChannelConfig) -> Self {
        let ChannelConfig { bot_id, kind } = value;
        match kind {
            ChannelKind::Group(name) => (
                Channel {
                    parent_id: Some(name),
                    ty: ChannelType::Group,
                    ..Default::default()
                },
                bot_id,
            ),
            ChannelKind::Private(name) => (
                Channel {
                    id: name,
                    ty: ChannelType::Private,
                    ..Default::default()
                },
                bot_id,
            ),
        }
    }
}

#[derive(Clone)]
struct AppState {
    channels: Vec<(Channel, String)>,
    client:   Client,
    secret:   String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (plugin, Initialize { config, .. }) = Plugin::new::<Config>().await.unwrap();

    let state = AppState {
        channels: config.channels.into_iter().map(<(Channel, String)>::from).collect(),
        client:   plugin.server.client(),
        secret:   config.secret,
    };

    let app: Router =
        Router::new().route("/webhook", axum::routing::post(webhook)).with_state(state);

    let listener = TcpListener::bind((config.host, config.port)).await?;

    let serve = axum::serve(listener, app);

    log::info!("Github notify plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
        _ = serve => {}
    }
    Ok(())
}
