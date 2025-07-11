use futures_util::StreamExt;
use serde::Deserialize;
use sithra_server::{
    routing::router::Router,
    server::{Server, ServerError},
    transport::{peer::Peer, util::FramedPeer},
};
use sithra_types::initialize::Initialize;
use thiserror::Error;
use tokio::task::JoinSet;

use crate::logger::init_log;

pub struct Plugin {
    peer:       Peer,
    pub server: Server,
    router:     Router,
}

impl Plugin {
    /// # Errors
    /// - [`PluginInitError::DeserializationError`] if the config could not be
    ///   deserialized.
    /// - [`PluginInitError::ConnectionClosed`] if the connection was closed
    ///   before the config was received.
    pub async fn new<Config>() -> Result<(Self, Initialize<Config>), PluginInitError>
    where
        Config: for<'de> Deserialize<'de>,
    {
        let peer = Peer::new();
        let server = Server::new();
        let router = Router::new();
        let mut framed = crate::transport::util::framed(peer);

        let init = loop {
            let Some(msg) = <FramedPeer as StreamExt>::next(&mut framed).await else {
                break Err(PluginInitError::ConnectionClosed);
            };
            if let Ok(msg) = msg {
                let is_init = msg.path.as_ref().is_some_and(|p| p == Initialize::<Config>::path());
                if is_init {
                    let config = msg.payload::<Initialize<Config>>();
                    break config.map_err(PluginInitError::DeserializationError);
                }
            }
        }?;

        init_log(server.client().sink());

        let peer = framed.into_inner();
        Ok((
            Self {
                peer,
                server,
                router,
            },
            init,
        ))
    }

    #[must_use]
    pub fn map<S, F>(self, f: F) -> Self
    where
        F: FnOnce(Router<S>) -> Router,
    {
        let Self {
            peer,
            server,
            router,
        } = self;
        Self {
            peer,
            server,
            router: f(router.with_state(())),
        }
    }

    pub async fn map_async<F, Fut>(self, f: F) -> Self
    where
        F: FnOnce(Router) -> Fut,
        Fut: Future<Output = Router>,
    {
        let Self {
            peer,
            server,
            router,
        } = self;
        Self {
            peer,
            server,
            router: f(router).await,
        }
    }

    #[must_use]
    pub fn run(self) -> JoinSet<Result<(), ServerError>> {
        let Self {
            peer,
            server,
            router,
        } = self;
        let (write, read) = peer.split();

        server.service(router).serve(write, read)
    }
}

#[derive(Debug, Error)]
pub enum PluginInitError {
    #[error("Failed to deserialize config")]
    DeserializationError(String),
    #[error("Connection closed")]
    ConnectionClosed,
}
