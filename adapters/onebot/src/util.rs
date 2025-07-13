use std::{net::IpAddr, time::Duration};

use base64::{Engine, prelude::BASE64_STANDARD};
use hyper::header::HeaderValue;
use serde::{Deserialize, Deserializer, Serialize};
use sithra_kit::server::response::Response;
use thiserror::Error;
use tokio::{fs, io::AsyncReadExt, sync::mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message as WsMessage, client::IntoClientRequest},
};
use ulid::Ulid;

use crate::{AdapterState, api::request::ApiCall};

pub(crate) fn de_str_from_num<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let num: i64 = Deserialize::deserialize(deserializer)?;
    Ok(num.to_string())
}

pub fn send_req<T: Serialize>(
    state: &AdapterState,
    id: Ulid,
    api_call: &ApiCall<T>,
    err: &str,
) -> Option<Response> {
    let req = serde_json::to_string(api_call);
    let req = match req {
        Err(se_err) => {
            log::error!("Failed to serialize {err} request: {se_err}");
            let mut response =
                Response::error(format!("Failed to serialize {err} request: {se_err}"));
            response.correlate(id);
            return Some(response);
        }
        Ok(req) => req,
    };
    let result = state.ws_tx.send(WsMessage::Text(req.into()));
    if let Err(ws_err) = result {
        log::error!("Failed to send {err} request: {ws_err}");
        let mut response = Response::error(format!("Failed to send {err} request: {ws_err}"));
        response.correlate(id);
        return Some(response);
    }
    None
}

/// Exponential backoff retry for async operations
/// # Errors
/// Returns an error if the maximum number of retries is exceeded.
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut f: F,
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    let mut retries = 0;

    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) if retries >= max_retries => {
                log::error!("Max retries ({max_retries}) exceeded. Last error: {err}");
                return Err(err);
            }
            Err(err) => {
                retries += 1;
                log::warn!("Attempt {retries}/{max_retries} failed: {err}. Retrying in {delay:?}");

                tokio::time::sleep(delay).await;

                // Exponential backoff with jitter
                delay = (delay * 2).min(max_delay);
                let jitter = Duration::from_millis(fastrand::u64(..) % 1000);
                delay = delay.saturating_add(jitter);
            }
        }
    }
}

/// Connection manager for WebSocket with automatic reconnection
pub struct ConnectionManager {
    ws_url:    String,
    token:     Option<String>,
    pub ws_tx: mpsc::UnboundedSender<WsMessage>,
}

impl ConnectionManager {
    #[must_use]
    pub fn new(
        ws_url: String,
        token: Option<String>,
    ) -> (Self, mpsc::UnboundedReceiver<WsMessage>) {
        let (ws_tx, ws_rx) = mpsc::unbounded_channel();

        let manager = Self {
            ws_url,
            token,
            ws_tx,
        };

        (manager, ws_rx)
    }

    /// Establish WebSocket connection with retry
    ///
    /// # Errors
    /// Returns an error if the WebSocket connection fails.
    pub async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        String,
    > {
        let mut request = self
            .ws_url
            .as_str()
            .into_client_request()
            .map_err(|e| format!("Failed to create WebSocket request: {e}"))?;

        if let Some(access_token) = &self.token {
            request.headers_mut().insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {access_token}"))
                    .map_err(|e| format!("Failed to create auth header: {e}"))?,
            );
        }

        retry_with_backoff(
            || async {
                connect_async(request.clone())
                    .await
                    .map(|(stream, _)| stream)
                    .map_err(|e| format!("WebSocket connection failed: {e}"))
            },
            7,                          // max retries
            Duration::from_millis(500), // initial delay
            Duration::from_secs(30),    // max delay
        )
        .await
    }

    /// Run the connection with automatic reconnection on failure
    pub async fn run_with_reconnect<F, Fut>(&self, mut handler: F)
    where
        F: FnMut(
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        ) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        loop {
            log::info!("Establishing WebSocket connection...");

            match self.connect().await {
                Ok(ws_stream) => {
                    log::info!("WebSocket connection established successfully");

                    // Run the handler with the connection
                    handler(ws_stream).await;

                    log::warn!("WebSocket connection closed, attempting to reconnect...");
                }
                Err(e) => {
                    log::error!("Failed to establish connection: {e}");
                }
            }

            // Wait before attempting reconnection
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

#[must_use]
pub fn is_loopback(url_str: &str) -> bool {
    let Ok(url) = url::Url::parse(url_str) else {
        return false;
    };

    let Some(host) = url.host_str() else {
        return false;
    };

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.is_loopback();
    }

    false
}

/// # Errors
/// - `ProcessError::IoError`: File IO error
/// - `ProcessError::FilePathError`: File path error
pub async fn or_in_base64(url_str: &str) -> Result<String, ProcessError> {
    let Ok(url) = url::Url::parse(url_str) else {
        return Ok(url_str.to_owned());
    };
    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|()| ProcessError::FilePathError)?;
        let mut buffer = Vec::new();
        {
            let mut file = fs::File::open(path).await?;
            file.read_to_end(&mut buffer).await?;
        }
        let base64 = BASE64_STANDARD.encode(buffer);
        Ok(format!("base64://{base64}"))
    } else {
        Ok(url_str.to_owned())
    }
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("File IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("File path error")]
    FilePathError,
}
