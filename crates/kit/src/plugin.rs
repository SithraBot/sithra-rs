use std::{env, process};

use futures_util::{SinkExt as _, StreamExt};
use serde::Deserialize;
use sithra_server::{
    routing::router::Router,
    server::{Server, ServerError},
    transport::{
        datapack::{DataPack, RequestDataPack},
        peer::Peer,
        util::FramedPeer,
    },
};
use sithra_types::initialize::{Initialize, InitializeResult, PluginInitError};
use tokio::task::JoinSet;

use crate::logger::init_log;

pub struct Plugin {
    peer:       Peer,
    pub server: Server,
    router:     Router,
}

fn handle_options(version: &str, name: &str) -> bool {
    handle_version(env::args(), version) || handle_name(env::args(), name)
}

fn handle_version(mut args: impl Iterator<Item = String>, version: &str) -> bool {
    let is_get_version = args.any(|arg| arg.trim().eq("--version"));
    if is_get_version {
        print!("{version}");
        return true;
    }
    false
}

fn handle_name(mut args: impl Iterator<Item = String>, name: &str) -> bool {
    let is_get_name = args.any(|arg| arg.trim().eq("--name"));
    if is_get_name {
        print!("{name}");
        return true;
    }
    false
}

impl Plugin {
    /// # Errors
    /// - [`PluginInitError::DeserializationError`] if the config could not be
    ///   deserialized.
    /// - [`PluginInitError::ConnectionClosed`] if the connection was closed
    ///   before the config was received.
    ///
    /// # Panics
    /// - If the initialization response fails to send.
    pub async fn new<Config>(version: &str, name: &str) -> (Self, Initialize<Config>)
    where
        Config: for<'de> Deserialize<'de>,
    {
        if handle_options(version, name) {
            process::exit(0);
        }
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
                    break config.map_err(PluginInitError::ConfigDeserializeError);
                }
            }
        };

        let init = match init {
            Ok(init) => init,
            Err(err) => {
                framed
                    .send(
                        DataPack::builder()
                            .path(Initialize::<Config>::path())
                            .build_with_payload(InitializeResult::Err(err)),
                    )
                    .await.ok();
                tokio::signal::ctrl_c().await.ok();
                process::exit(1);
            }
        };

        init_log(server.client().sink());

        server
            .client()
            .send(
                RequestDataPack::default()
                    .path(Initialize::<Config>::path())
                    .payload(InitializeResult::Ok(())),
            )
            .unwrap_or_else(|_| panic!("Failed to send initialization response: [{name}]"));

        let peer = framed.into_inner();
        (
            Self {
                peer,
                server,
                router,
            },
            init,
        )
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

#[macro_export]
macro_rules! plugin {
    ($ty:ty) => {
        $crate::plugin::Plugin::new::<$ty>(env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME")).await
    };
    () => {
        $crate::plugin::Plugin::new::<()>(env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME")).await
    };
}
