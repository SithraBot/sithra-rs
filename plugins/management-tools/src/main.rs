use std::{
    num::{IntErrorKind, ParseIntError},
    time::Duration,
};

use serde::Deserialize;
use sithra_kit::{
    plugin::Plugin,
    server::{
        extract::{
            context::{Clientful, Context},
            payload::Payload,
        },
        router,
        server::Client,
    },
    transport::channel::Channel,
    types::{
        channel::ContextExt as _,
        initialize::Initialize,
        message::{Message, SendMessage, common::CommonSegment as H},
        smsg,
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
    let (plugin, Initialize { config, .. }) = Plugin::new::<Config>().await.unwrap();

    let client = plugin.server.client();

    let state = AppState {
        admins: Arc::new(config.admins),
        client,
    };

    let plugin = plugin.map(move |r| {
        router! {r =>
            Message[channelinfo, mute]
        }
        .with_state(state)
    });

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
                return Some(smsg!(concat!($action, "失败喵，请通过错误日志查看具体信息喵")).into());
            }
        }
    };
}

async fn channelinfo(Payload(msg): Payload<Message<H>>, channel: Channel) -> Option<SendMessage> {
    match msg.content.as_slice() {
        [H::Text(text)] if text.trim() == "channelinfo" => {}
        _ => {
            return None;
        }
    }
    let Channel {
        id,
        ty,
        name,
        parent_id,
        self_id: _,
    } = channel;
    let info = format!(
        "频道ID: {}\n频道类型: {}\n频道名称: {}\n父频道ID: {}",
        id,
        ty,
        name,
        parent_id.unwrap_or_else(|| "无".to_owned()),
    );
    Some(smsg!(info))
}

async fn mute(ctx: Context<Message<H>, AppState>) -> Option<SendMessage> {
    let args = parse_cmd(&ctx.content);
    let channel = ctx.request.channel()?;
    let (id, duration) = match args {
        Ok(ok) => ok,
        Err(ParseErr::InvalidNumber) => return Some(smsg!("无效的数字喵")),
        Err(ParseErr::NotEnoughArgs) => {
            return Some(smsg!("需要俩参数喵，用户ID和时长喵"));
        }
        Err(ParseErr::NotMatch) => return None,
    };

    if channel.parent_id.is_none() {
        return Some(smsg!("只能在群聊中使用喵"));
    }

    if !auth(&channel.id, &ctx.state.admins) {
        return Some(smsg!("你没有权限喵"));
    }

    let is_unmute = duration.is_zero();

    let res = ctx.set_mute_member(id, duration).await;
    tap_err!(res, "禁言");
    Some(smsg!(H [
        text: if is_unmute {"解禁成功喵 "} else {"禁言成功喵 "},
        at: id,
        text: if is_unmute {" 😎堂堂复活喵"} else {" 💀"},
    ]))
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
