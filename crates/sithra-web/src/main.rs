use std::net::{IpAddr, Ipv4Addr};

use axum::Router;
use sithra::{conf, loader};
use tokio::signal;
use tower_http::services::ServeDir;

use crate::util::{addr, addr_display};

mod util;

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = conf::Config::load_config("./config.toml");
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Err(err.into());
        }
    };
    let mut loader = loader::Loader::new(config);
    let errs = loader.load_all().await;
    for (name, err) in errs {
        log::error!("Failed to load plugin {name}: {err}");
    }

    let router = Router::new().fallback_service(ServeDir::new("web"));
    let addr = addr((DEFAULT_HOST, DEFAULT_PORT));
    let server = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let server = axum::serve(listener, router);
            log::info!("Server started on {}", addr_display(addr));
            Some(tokio::spawn(server.into_future()))
        }
        Err(err) => {
            log::error!("Failed to bind to address: {err}");
            None
        }
    };

    signal::ctrl_c().await?;

    loader.abort_all();
    if let Some(f) = server {
        f.abort();
    }
    Ok(())
}
