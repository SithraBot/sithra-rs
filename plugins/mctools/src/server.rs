use base64::{Engine, prelude::BASE64_STANDARD};
use bytes::Bytes;
use resvg::{
    render, tiny_skia,
    usvg::{Options, Tree},
};
use serde::{Deserialize, Serialize};
use sithra_kit::{
    server::extract::{botid::BotId, payload::Payload, state::State},
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use thiserror::Error;

use crate::{
    AppState,
    util::{cmd, truncate_tail},
};

const CARD_SVG_TEMPLATE: &str = include_str!("../static/mc-server.svg");

pub struct Template {
    pub the_server_ip:    String,
    pub the_server_port:  String,
    pub online:           String,
    pub game_version:     String,
    pub protocol_version: String,
    pub server_type:      String,
    pub info:             String,
    pub image_url:        Option<String>,
}

pub fn apply_template(template: &Template) -> String {
    let mut svg = CARD_SVG_TEMPLATE.to_owned();
    svg = svg.replace("{{the_server_ip}}", &template.the_server_ip);
    svg = svg.replace("{{the_server_port}}", &template.the_server_port);
    svg = svg.replace("{{online}}", &template.online);
    svg = svg.replace("{{game_version}}", &template.game_version);
    svg = svg.replace("{{protocol_version}}", &template.protocol_version);
    svg = svg.replace("{{server_type}}", &template.server_type);
    svg = svg.replace("{{info}}", &truncate_tail(template.info.trim(), 20));
    if let Some(image_url) = &template.image_url {
        svg = svg.replace("{{image_url}}", image_url);
        svg = svg.replace("{{image_display}}", "block");
        svg = svg.replace("{{icon_display}}", "none");
    } else {
        svg = svg.replace("{{image_display}}", "none");
        svg = svg.replace("{{icon_display}}", "inline");
    }
    svg
}

#[derive(Deserialize, Serialize)]
pub struct ApiInfo {
    ip:       String,
    port:     u16,
    version:  String,
    protocol: Option<ApiInfoProtocol>,
    icon:     Option<String>,
    software: Option<String>,
    motd:     ApiInfoMotd,
    players:  ApiInfoPlayers,
}

#[derive(Deserialize, Serialize)]
pub struct ApiInfoProtocol {
    version: u16,
}

#[derive(Deserialize, Serialize)]
pub struct ApiInfoMotd {
    clean: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ApiInfoPlayers {
    online: u32,
    max:    u32,
}

impl From<ApiInfo> for Template {
    fn from(value: ApiInfo) -> Self {
        let ApiInfo {
            ip,
            port,
            version,
            protocol,
            icon,
            software,
            motd,
            players,
        } = value;
        let protocol_version = if let Some(protocol) = protocol {
            protocol.version.to_string()
        } else {
            "未知".to_owned()
        };
        let software = if let Some(software) = software {
            software
        } else {
            "不知道喵".to_owned()
        };
        Self {
            the_server_ip: ip,
            the_server_port: port.to_string(),
            online: format!("{}/{}", players.online, players.max),
            game_version: version,
            protocol_version,
            server_type: software,
            info: motd.clean.join(";"),
            image_url: icon,
        }
    }
}

fn render_svg(svg: &str, options: &Options) -> Result<Bytes, RenderError> {
    let tree = Tree::from_str(svg, options)?;

    let size = tree.size().to_int_size();
    let (width, height) = (size.width(), size.height());

    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).ok_or(RenderError::FailedToCreatePixmap)?;

    render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let buffer = pixmap.encode_png()?;

    Ok(Bytes::from(buffer))
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("SVG parsing error: {0}")]
    ParsingError(#[from] resvg::usvg::Error),
    #[error("Failed to create pixmap")]
    FailedToCreatePixmap,
    #[error("Failed to encode PNG")]
    EncodingError(#[from] png::EncodingError),
}

pub async fn mcserver(
    Payload(message): Payload<Message<H>>,
    State(state): State<AppState<'_>>,
    BotId(bot_id): BotId,
) -> Option<SendMessage> {
    let AppState {
        svg_options,
        client,
    } = state;
    let cmd = cmd(&message.content);
    let ip = cmd.strip_prefix("mcserver ")?.trim();

    let bot = bot_id.unwrap_or_else(|| "UnknownBotId".to_owned());

    let req_future = client
        .get(format!("https://api.mcsrvstat.us/3/{ip}"))
        .header("User-Agent", format!("SithraBot;McTools;{bot}"))
        .send();

    let Ok(response) = req_future.await else {
        return Some("请求失败喵".into());
    };

    let status = response.status();

    let info = match response.json::<ApiInfo>().await {
        Ok(info) => info,
        Err(err) => {
            log::error!("Failed to parse API response: {err}, status: {status}");
            return Some("响应解析失败喵，可能是服务器状态异常喵".into());
        }
    };

    let template: Template = info.into();

    let img = render_svg(&apply_template(&template), &svg_options);

    let Ok(img) = img else {
        return Some("卡片渲染失败喵".into());
    };

    let base64 = BASE64_STANDARD.encode(img);
    let img = format!("base64://{base64}");

    Some(msg!(H[img: img]).into())
}
