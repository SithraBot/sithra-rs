use std::{
    num::{IntErrorKind, ParseIntError},
    time::Duration,
};

use serde::Deserialize;
use sithra_kit::{
    plugin::Plugin,
    server::{
        extract::context::{Clientful, Context},
        server::Client,
    },
    transport::channel::Channel,
    types::{
        channel::SetMute,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use triomphe::Arc;

#[derive(Debug, Clone, Default, Deserialize)]
struct Config {
    #[serde(default)]
    admins: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    admins: Arc<Vec<String>>,
    client: Client,
}

impl Clientful for AppState {
    fn client(&self) -> &Client {
        &self.client
    }
}

#[tokio::main]
async fn main() {
    let (plugin, config) = Plugin::<Config>::new().await.unwrap();

    let client = plugin.server.client();

    let state = AppState {
        admins: Arc::new(config.admins),
        client,
    };

    let plugin = plugin.map(move |r| r.route_typed(Message::on(mute)).with_state(state));

    log::info!("Management Tools plugin started");

    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

macro_rules! tap_err {
    ($val:expr, $action:expr) => {
        match $val {
            Ok(ok) => ok,
            Err(err) => {
                log::error!(concat!("Failed to ", $action, ": {:?}"), err);
                return Some(msg!(H[text: concat!($action, "失败喵，请通过错误日志查看具体信息喵")]).into());
            }
        }
    };
}

async fn mute(ctx: Context<Message<H>, AppState>, mut channel: Channel) -> Option<SendMessage> {
    let args = parse_cmd(&ctx.content);
    let (id, duration) = match args {
        Ok(ok) => ok,
        Err(ParseErr::InvalidNumber) => return Some(msg!(H[text: "无效的数字喵"]).into()),
        Err(ParseErr::NotEnoughArgs) => {
            return Some(msg!(H[text: "需要俩参数喵，用户ID和时长喵"]).into());
        }
        Err(ParseErr::NotMatch) => return None,
    };

    if channel.parent_id.is_none() {
        return Some(msg!(H[text: "只能在群聊中使用喵"]).into());
    }

    if !auth(&channel.id, &ctx.state.admins) {
        return Some(msg!(H[text: "你没有权限喵"]).into());
    }

    let is_unmute = duration.is_zero();

    id.clone_into(&mut channel.id);

    let set_mute = SetMute { channel, duration };
    let res = ctx.post(set_mute);
    let res = tap_err!(res, "禁言").await;
    tap_err!(res, "禁言");
    Some(
        msg!(H [
            text: if is_unmute {"解禁成功喵 "} else {"禁言成功喵 "},
            at: id,
            text: if is_unmute {" 😎堂堂复活喵"} else {" 💀"},
        ])
        .into(),
    )
}

fn auth(user: &String, admins: &[String]) -> bool {
    admins.contains(user)
}

fn parse_cmd(segs: &[H]) -> Result<(&str, Duration), ParseErr> {
    match segs {
        [H::Text(cmd), H::At(user_id), H::Text(duration)] if cmd.trim() == "mute" => {
            let duration = duration.trim().parse()?;
            Ok((user_id, Duration::from_secs(duration)))
        }
        [H::Text(cmd)] => {
            let Some(args) = cmd.strip_prefix("mute ") else {
                return Err(ParseErr::NotMatch);
            };
            let args = args.split_whitespace().collect::<Vec<_>>();
            if args.len() != 2 {
                return Err(ParseErr::NotEnoughArgs);
            }
            let user_id = args[0];
            let duration = args[1].parse()?;
            Ok((user_id, Duration::from_secs(duration)))
        }
        _ => Err(ParseErr::NotMatch),
    }
}

enum ParseErr {
    InvalidNumber,
    NotEnoughArgs,
    NotMatch,
}

impl From<ParseIntError> for ParseErr {
    fn from(e: ParseIntError) -> Self {
        match e.kind() {
            IntErrorKind::Empty => Self::NotEnoughArgs,
            _ => Self::InvalidNumber,
        }
    }
}
