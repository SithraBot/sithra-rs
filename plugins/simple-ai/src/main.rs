use std::collections::VecDeque;

use ahash::AHashMap;
use rig::{
    agent::Agent,
    client::CompletionClient,
    completion::Chat,
    providers::openrouter::{self, CompletionModel},
};
use serde::Deserialize;
use sithra_kit::{
    plugin,
    server::extract::{payload::Payload, state::State},
    transport::channel::Channel,
    types::{
        initialize::Initialize,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use tokio::sync::Mutex;
use triomphe::Arc;

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "api-key")]
    api_key:     String,
    #[serde(rename = "base-url")]
    #[serde(default = "default_base_url")]
    base_url:    String,
    #[serde(default = "default_model")]
    model:       String,
    #[serde(default = "default_preamble")]
    preamble:    String,
    temperature: Option<f64>,
    context:     Option<Vec<String>>,
    #[serde(default = "default_max_history")]
    #[serde(rename = "max-history")]
    max_history: usize,
}
fn default_base_url() -> String {
    "https://openrouter.ai/api/v1".to_owned()
}
fn default_model() -> String {
    "openai/gpt-5-chat".to_owned()
}
fn default_preamble() -> String {
    "You are a helpful assistant.".to_owned()
}
const fn default_max_history() -> usize {
    20
}

#[derive(Clone)]
struct AppState {
    agent:       Arc<Agent<CompletionModel>>,
    history:     Arc<Mutex<AHashMap<String, VecDeque<rig::message::Message>>>>,
    max_history: usize,
}

const MESSAGE_FORMAT_PREAMBLE: &str =
    "You are a helpful ROLEPLAY AI assistant integrated into a roleplay chat service. You will \
     receive messages from users, and each message will be prefixed with the name of the user \
     it originated from, in the format `[user_name]: message`. When you respond, do not add \
     any prefix to your own message.";

#[tokio::main]
async fn main() {
    let (mut plugin, Initialize { config, .. }) = plugin!(Config);
    let ai_client = openrouter::Client::builder(&config.api_key).base_url(&config.base_url).build();
    let ai_client = plugin.expect(ai_client).await;
    let mut agent = ai_client
        .agent(&config.model)
        .append_preamble(MESSAGE_FORMAT_PREAMBLE)
        .append_preamble(&config.preamble);
    if let Some(temperature) = config.temperature {
        agent = agent.temperature(temperature);
    }
    if let Some(context) = config.context {
        for item in context {
            agent = agent.context(&item);
        }
    }
    let agent = agent.build();
    let state = AppState {
        agent:       Arc::new(agent),
        history:     Arc::new(Mutex::new(AHashMap::new())),
        max_history: config.max_history,
    };
    let plugin = plugin.map(|r| r.route_typed(Message::on(ai)).with_state(state));
    log::info!("Simple AI started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn ai(
    Payload(msg): Payload<Message<H>>,
    channel: Channel,
    State(AppState {
        agent,
        history,
        max_history,
    }): State<AppState>,
) -> Option<SendMessage> {
    // log::debug!("{:?}", msg.content);
    let msg = match msg.content.as_slice() {
        [H::At(id), ..] if channel.self_id.as_ref().is_some_and(|sid| sid.eq(id)) => {
            text_only(&msg)
        }
        [H::Text(f)] => f.strip_prefix("!?").map(ToOwned::to_owned),
        _ => None,
    }?;
    let msg = format!("[{}]: {msg}", channel.name);
    log::info!("Received message: {msg}");
    let key = if let Some(id) = channel.parent_id {
        id
    } else if let Some(id) = channel.self_id {
        id
    } else {
        "global".to_owned()
    };
    let current_history = history
        .lock()
        .await
        .entry(key.clone())
        .or_insert_with(VecDeque::new)
        .iter()
        .map(Clone::clone)
        .collect();
    let response = agent.chat(msg.trim(), current_history).await;
    let response = match response {
        Ok(res) => res,
        Err(err) => {
            log::error!("{err}");
            return Some(msg!("[API错误]"));
        }
    };
    history.lock().await.entry(key.clone()).and_modify(|h| {
        h.push_back(rig::message::Message::user(msg));
        h.push_back(rig::message::Message::assistant(&response));
    });
    shift_history(history.lock().await.entry(key).or_default(), max_history);
    log::info!("Received response: {response:?}");
    Some(msg!(response))
}

fn shift_history(history: &mut VecDeque<rig::message::Message>, max_history: usize) {
    while history.len() > max_history {
        history.pop_front();
    }
    if history
        .front()
        .is_some_and(|m| matches!(m, rig::message::Message::Assistant { .. }))
    {
        history.pop_front();
    }
}

fn text_only(msg: &[H]) -> Option<String> {
    let raw = msg
        .iter()
        .filter_map(|h| h.text_opt().map(|s| s.trim()))
        .fold(String::new(), |s, h| s + h);
    if raw.trim().is_empty() {
        None
    } else {
        Some(raw)
    }
}
