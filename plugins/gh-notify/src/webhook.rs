use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sithra_kit::{transport::datapack::RequestDataPack, types::smsg};

use crate::{AppState, event::GithubPushEvent};

type HmacSha256 = Hmac<Sha256>;

macro_rules! tap_err {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                log::error!("Error: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    };
}

pub async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let signature = if let Some(sig) = headers.get("x-hub-signature-256") {
        match sig.to_str() {
            Ok(s) => s,
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    } else {
        log::warn!("Expected 'x-hub-signature-256' header, but got none");
        return StatusCode::BAD_REQUEST;
    };
    let mut mac = match HmacSha256::new_from_slice(state.secret.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to create HMAC: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    mac.update(&body);
    let result = mac.finalize();
    let expected_signature = format!("sha256={}", hex::encode(result.into_bytes()));
    if !signature.eq_ignore_ascii_case(&expected_signature) {
        log::warn!("Invalid signature: expected {expected_signature}, got {signature}",);
        return StatusCode::UNAUTHORIZED;
    }

    let event_type = if let Some(event) = headers.get("x-github-event") {
        match event.to_str() {
            Ok(e) => e,
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    } else {
        log::warn!("Expected 'x-github-event' header, but got none");
        return StatusCode::BAD_REQUEST;
    };

    if event_type == "push" {
        tap_err!(handle_push(state, body).await);
    }
    
    StatusCode::OK
}

pub async fn handle_push(state: AppState, body: Bytes) -> anyhow::Result<()> {
    let payload: GithubPushEvent = serde_json::from_slice(&body)?;
    let send_msg = smsg!(payload.to_string());
    let req = RequestDataPack::default().payload(send_msg);
    for (channel, bot_id) in state.channels {
        let req = req.clone().bot_id(bot_id).channel(channel);
        state.client.post(req)?.await?;
    }
    Ok(())
}
