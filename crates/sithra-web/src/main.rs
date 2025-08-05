use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, get_service, post},
};
use clap::Parser as _;
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use sithra::{conf, loader};
use tokio::{signal, sync::RwLock};
use tower_http::services::{ServeDir, ServeFile};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};
use triomphe::Arc;

use crate::{
    auth::{Auth, AuthError, CREDENTIALS, Claims, KEYS, auth_verify, flush_credentials},
    util::{Args, addr_display},
};

mod auth;
mod util;

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 8080;

#[derive(Clone)]
struct AppState {
    loader: Arc<RwLock<loader::Loader>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_env_var("SITHRA_LOG")
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt::layer().pretty())
        .init();
    let args = Args::parse();
    let config = conf::Config::load_config("./config.toml", "./config.d");
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Err(err.into());
        }
    };
    let loader = loader::Loader::new(config);
    let errs = loader.load_all().await;
    for (name, err) in errs {
        log::error!("Failed to load plugin {name}: {err}");
    }
    let state = AppState {
        loader: Arc::new(RwLock::new(loader)),
    };

    let mut router = Router::new()
        .route("/api/plgs_info", get(plgs_info))
        .route("/api/plg_details/{*id}", get(plg_details))
        .route("/api/save_config", post(save_config))
        .route("/api/ctrl_plg", post(ctrl_plg))
        .route("/api/del_plg/{*id}", delete(del_plg))
        .route("/api/clone_plg", post(clone_plg))
        .route("/api/list_files", post(list_files))
        .route("/auth/is_registered", get(is_registered))
        .route("/auth/register", post(register))
        .route("/auth/authorize", post(authorize))
        .route("/auth/verify", get(verify));
    if !args.api_only() {
        let path = std::path::Path::new(&args.web_path("web")).to_owned();
        let index = path.join("index.html");
        let app_dir = path.join("_app");
        let html404 = path.join("404.html");
        let icon = path.join("icon.png");
        router = router
            .nest_service("/_app", get_service(ServeDir::new(app_dir)))
            .route("/icon.png", get_service(ServeFile::new(icon)))
            .fallback(get_service(ServeFile::new(index)).fallback_service(ServeFile::new(html404)));
    }
    let router = router.with_state(state);
    let addr = args.addr((DEFAULT_HOST, DEFAULT_PORT));
    let server = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let server = axum::serve(listener, router);
            log::info!(
                port = addr.0.to_string(),
                host = addr.1.to_string();
                "Server started on {}", addr_display(addr)
            );
            Some(tokio::spawn(server.into_future()))
        }
        Err(err) => {
            log::error!("Failed to bind to address: {err}");
            None
        }
    };

    signal::ctrl_c().await?;

    if let Some(f) = server {
        f.abort();
    }
    Ok(())
}

async fn is_registered() -> Json<bool> {
    Json(CREDENTIALS.read().unwrap().is_some())
}

async fn register(Json(request): Json<Claims>) -> Result<String, AuthError> {
    if request.hex.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    let token = jsonwebtoken::encode(&Header::default(), &request, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    {
        let mut credentials = CREDENTIALS.write().unwrap();
        if credentials.is_some() {
            return Err(AuthError::CredentialsAlreadyExists);
        }
        *credentials = Some(request.hex);
    }
    flush_credentials().await?;
    Ok(token)
}

async fn authorize(Json(request): Json<Claims>) -> Result<String, AuthError> {
    if request.hex.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    auth_verify(&request)?;
    let token = jsonwebtoken::encode(&Header::default(), &request, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(token)
}

async fn verify(_auth: Auth) -> &'static str {
    "ok"
}

async fn plgs_info(State(state): State<AppState>) -> impl IntoResponse {
    log::debug!("GET /api/plgs_info");
    let plugins = state.loader.read().await.plugins();
    (StatusCode::OK, Json(plugins))
}

async fn plg_details(
    _auth: Auth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    log::debug!(id; "GET /api/plg_details/{id}");
    let details = state.loader.read().await.plugin_details(&id).await;
    (
        if details.is_some() {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        },
        Json(details),
    )
}

#[derive(Deserialize)]
struct SaveConfig {
    id:     String,
    config: String,
}

macro_rules! tap_err {
    ($res:expr) => {
        if let Err(err) = $res {
            log::error!("{err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}"));
        }
    };
    (@tap; $res:expr) => {
        match $res {
            Ok(v) => (v),
            Err(err) => {
                log::error!("{err}");
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response();
            }
        }
    };
}

async fn save_config(
    _auth: Auth,
    State(state): State<AppState>,
    Json(request): Json<SaveConfig>,
) -> impl IntoResponse {
    let SaveConfig { id, config } = request;
    log::debug!(id; "POST /api/save_config for [{id}]");
    let res = state.loader.write().await.config.set_config(&id, &config);
    tap_err!(res);
    let res = state.loader.read().await.config.flush_raw(&id).await;
    tap_err!(res);
    log::info!("[{id}] config saved");
    let res = {
        let loader = state.loader.read().await;
        loader.abort(&id);
        loader.load(&id).await
    };
    match res {
        Err(err) => {
            log::error!("[{id}] load failed: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}"));
        }
        Ok(true) => log::info!("[{id}] restarted"),
        _ => {}
    }
    (StatusCode::OK, String::from("ok"))
}

#[derive(Deserialize)]
struct CtrlPlugin {
    id:     String,
    enable: bool,
}

async fn ctrl_plg(
    _auth: Auth,
    State(state): State<AppState>,
    Json(request): Json<CtrlPlugin>,
) -> impl IntoResponse {
    let CtrlPlugin { id, enable } = request;
    log::debug!(id; "POST /api/ctrl_plg for [{id}]");
    state.loader.write().await.config.set_enable(&id, enable);
    if enable {
        let res = state.loader.read().await.load(&id).await;
        tap_err!(res);
    } else {
        state.loader.read().await.abort(&id);
    }
    let res = state.loader.read().await.config.flush_base().await;
    tap_err!(res);
    (StatusCode::OK, String::from("ok"))
}

async fn del_plg(
    _auth: Auth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = state.loader.write().await.config.delete_file(&id).await;
    tap_err!(res);
    state.loader.write().await.config.remove(&id);
    state.loader.read().await.abort(&id);
    let res = state.loader.read().await.config.flush_base().await;
    tap_err!(res);
    (StatusCode::OK, String::from("ok"))
}

#[derive(Deserialize)]
struct ClonePlugin {
    id: String,
    to: String,
}

async fn clone_plg(
    _auth: Auth,
    State(state): State<AppState>,
    Json(request): Json<ClonePlugin>,
) -> impl IntoResponse {
    let ClonePlugin { id, to } = request;
    state.loader.write().await.config.duplicate(&id, &to);
    tap_err!(state.loader.read().await.config.flush_base().await);
    tap_err!(state.loader.read().await.config.flush_raw(&to).await);
    (StatusCode::OK, String::from("ok"))
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    name: String,
    ty:   FileType,
}

#[derive(Serialize)]
enum FileType {
    File,
    Dir,
}

#[derive(Deserialize)]
struct ListFiles {
    path: String,
}

async fn list_files(_auth: Auth, Json(request): Json<ListFiles>) -> Response {
    let ListFiles { path } = request;
    let path = PathBuf::from(path);
    if path.is_dir() {
        let mut entries = tap_err!(@tap;tokio::fs::read_dir(path).await);
        let mut entries_ = Vec::<FileEntry>::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            entries_.push(FileEntry {
                path: entry.path().to_str().unwrap_or("").to_owned(),
                name: entry.file_name().to_str().unwrap_or("").to_owned(),
                ty:   if tap_err!(@tap;entry.file_type().await).is_dir() {
                    FileType::Dir
                } else {
                    FileType::File
                },
            });
        }
        (StatusCode::OK, Json(entries_)).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json("not a directory")).into_response()
    }
}
