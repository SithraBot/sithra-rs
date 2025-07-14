use sithra_kit::{
    plugin::Plugin,
    server::{
        extract::{payload::Payload, state::State},
        router,
    },
    types::{
        initialize::Initialize,
        message::{Message, Segments, SendMessage, common::CommonSegment as H},
        smsg,
    },
};

use crate::base::BaseXMap;

mod base;

#[derive(Clone)]
struct AppState {
    map: BaseXMap,
}

#[tokio::main]
async fn main() {
    let (plugin, Initialize { config, .. }) = Plugin::new::<BaseXMap>().await.unwrap();
    let state = AppState { map: config };
    let plugin = plugin.map(move |r| {
        router!(r =>
            Message [
                encrypt,
                decrypt
            ]
        )
        .with_state(state)
    });
    log::info!("Crypt plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn encrypt(
    Payload(msg): Payload<Message<H>>,
    State(state): State<AppState>,
) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("encrypt ")?.to_owned();
    let Message { mut content, .. } = msg;
    {
        let first = content.first_mut()?;
        *first = H::text(&text);
    }
    let content: Segments<_> = content
        .into_iter()
        .map(|seg| {
            if let H::Text(v) = seg {
                if v.is_empty() {
                    return H::text(v);
                }
                H::text(state.map.encode(v.as_bytes()))
            } else {
                seg
            }
        })
        .collect();
    Some(smsg!(content))
}

async fn decrypt(
    Payload(msg): Payload<Message<H>>,
    State(state): State<AppState>,
) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("decrypt ")?.to_owned();
    let Message { mut content, .. } = msg;
    {
        let first = content.first_mut()?;
        *first = H::text(&text);
    }
    let content: Segments<_> = content
        .into_iter()
        .map(|seg| {
            if let H::Text(v) = seg {
                let decrypted = state.map.decode_string(v.trim());
                match decrypted {
                    Ok(decrypted) => H::text(decrypted),
                    Err(err) => H::text(err.to_string()),
                }
            } else {
                seg
            }
        })
        .collect();
    Some(smsg!(content))
}
