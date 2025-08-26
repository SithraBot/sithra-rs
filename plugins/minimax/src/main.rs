use base64::{Engine, prelude::BASE64_STANDARD};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use sithra_adapter_onebot::message::OneBotSegment as OH;
use sithra_kit::{
    matchopt, plugin,
    server::{
        extract::{payload::Payload, state::State},
        on,
    },
    types::{
        initialize::Initialize,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

#[derive(Deserialize, Clone)]
struct Config {
    #[serde(rename = "api-key")]
    api_key: String,
    #[serde(default = "default_speed")]
    speed:   f32,
    #[serde(default = "default_pitch")]
    pitch:   i32,
    #[serde(default = "default_vol")]
    vol:     u32,
    intensity: Option<i32>,
    timbre: Option<i32>,
    #[serde(rename = "sound-effects")]
    sound_effects: Option<String>,
    voice:   Vec<Voice>,
}

const fn default_speed() -> f32 {
    1.0
}

const fn default_pitch() -> i32 {
    0
}

const fn default_vol() -> u32 {
    1
}

#[derive(Deserialize, Clone)]
struct Voice {
    id:     String,
    weight: u8,
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    config: Config,
}

#[tokio::main]
async fn main() {
    let (mut plugin, Initialize { config, .. }) = plugin!(Config);
    let mut headers = HeaderMap::new();
    headers.append(
        "Content-Type",
        plugin.expect("application/json".parse()).await,
    );
    let state = AppState {
        client: plugin.expect(reqwest::Client::builder().default_headers(headers).build()).await,
        config,
    };
    let plugin = plugin.map(|r| {
        r.route_typed(Message::on(vc))
            .route("/minimax/voice_clone_api", on(vc_api))
            .with_state(state)
    });
    log::info!("Minimax started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

fn build_request(config: &Config, content: &str) -> serde_json::Value {
    let voice_map = config.voice
        .iter()
        .map(|voice| json!({"voice_id": voice.id, "weight": voice.weight}))
        .collect::<Vec<_>>();
    json!({
      "model": "speech-2.5-hd-preview",
      "text": content,
      "timber_weights": voice_map,
      "voice_setting": {
        "voice_id": "",
        "speed": config.speed,
        "pitch": config.pitch,
        "vol": config.vol,
        "latex_read": true
      },
      "audio_setting": {
        "sample_rate": 32000,
        "bitrate": 128_000,
        "format": "mp3"
      },
      "voice_modify": {
          "intensity": config.intensity,
          "timbre": config.timbre,
          "sound_effects": config.sound_effects,
      },
      "language_boost": "auto"
    })
}

#[derive(Deserialize)]
struct Response {
    data: ResponseData,
}

#[derive(Deserialize)]
struct ResponseData {
    audio: String,
}

macro_rules! tap_err {
    ($ident:ident = $expr:expr) => {
        let Ok($ident) = $expr else {
            return Some(msg!("[API 错误]"));
        };
    };
}

async fn vc(Payload(msg): Payload<Message<H>>, state: State<AppState>) -> Option<SendMessage> {
    let msg = matchopt!(msg.as_slice(), [H::Text(txt)] => txt)?;
    let content = msg.strip_prefix("vc ")?.trim();
    tap_err!(base64 = vc_api(Payload(content.to_owned()), state).await);
    let base64 = base64.0;
    log::info!("语音合成完成，合成结果正在发送");
    Some(msg!(OH[record: format!("base64://{base64}")]))
}

async fn vc_api(
    Payload(content): Payload<String>,
    State(state): State<AppState>,
) -> anyhow::Result<Payload<String>> {
    log::info!("开始语音合成: {content}");
    let request = build_request(&state.config, &content);
    let res = state
        .client
        .post("https://api.minimaxi.com/v1/t2a_v2")
        .body(serde_json::to_string(&request)?)
        .bearer_auth(&state.config.api_key)
        .send()
        .await?;
    let res = res.json::<Response>().await?;
    let base64 = hex2base64(&res.data.audio)?;
    Ok(Payload(base64))
}

fn hex2base64(input: &str) -> Result<String, hex::FromHexError> {
    hex::decode(input).map(|bytes| BASE64_STANDARD.encode(bytes))
}
