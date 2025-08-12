use std::{
    ffi::OsStr,
    fmt::Display,
    fs, io,
    process::Stdio,
    sync::{Arc, RwLock, Weak},
};

use ahash::HashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sithra_kit::{
    transport::{
        self, ValueError,
        datapack::{DataPack, DataPackCodec, DataPackCodecError},
        peer::{Peer, Reader, Writer},
    },
    types::{
        initialize::{Initialize, InitializeResult, PluginInitError},
        log::Log,
    },
};
use thiserror::Error;
use tokio::{process::Command, sync::broadcast, task::JoinHandle};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::conf::{BaseConfig, Config};

type JoinMap = Arc<RwLock<HashMap<String, (JoinHandle<()>, JoinHandle<()>)>>>;
type JoinMapWeak = Weak<RwLock<HashMap<String, (JoinHandle<()>, JoinHandle<()>)>>>;

pub struct Loader {
    // dirty:         watch::Sender<bool>,
    // clean_loop:    JoinHandle<()>,
    pub config:    Config,
    broadcast_tx:  broadcast::Sender<DataPack>,
    _broadcast_rx: broadcast::Receiver<DataPack>,
    join_map:      JoinMap,
}

#[derive(Clone)]
struct Entry {
    join_map: JoinMapWeak,
    key:      String,
}

impl Entry {
    const fn new(join_map: JoinMapWeak, key: String) -> Self {
        Self { join_map, key }
    }

    fn abort(self) {
        let Some(map) = self.join_map.upgrade() else {
            return;
        };
        let Some((jh1, jh2)) = map.write().unwrap().remove(&self.key) else {
            return;
        };
        jh1.abort();
        jh2.abort();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    id:      String,
    running: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginDetails {
    id:       String,
    name:     String,
    version:  String,
    #[serde(flatten)]
    config:   BaseConfig,
    toml_str: Option<String>,
    running:  bool,
}

impl Loader {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(32);
        let join_map = Arc::new(RwLock::new(HashMap::default()));
        // let (tx, rx) = watch::channel(false);

        Self {
            //     dirty: tx.clone(),
            //     clean_loop: tokio::spawn(Self::clean_loop(Arc::downgrade(&join_map), rx, tx)),
            config,
            broadcast_tx,
            _broadcast_rx: broadcast_rx,
            join_map,
        }
    }

    /// # Panics
    /// Panics if the loader is dropped while plugins are still running.
    #[must_use]
    pub fn plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();
        let join_map = self.join_map.read().unwrap();
        for (id, _) in self.config.iter() {
            plugins.push(PluginInfo {
                id:      id.to_owned(),
                running: join_map.contains_key(id),
            });
        }
        plugins
    }

    /// Returns the details of a plugin.
    /// # Panics
    /// Panics if the lock is poisoned.
    pub async fn plugin_details(&self, id: &str) -> Option<PluginDetails> {
        let config = self.config.get(id)?;
        let name = Command::new(&config.path).arg("--name").output().await.ok()?;
        if !name.status.success() {
            return None;
        }
        let name = String::from_utf8(name.stdout).ok()?;
        let version = Command::new(&config.path).arg("--version").output().await.ok()?;
        if !version.status.success() {
            return None;
        }
        let doc = if let Some(ref raw_config) = config.raw_config {
            Some(raw_config.to_string())
        } else {
            self.config.doc.get(id).and_then(|v| v.get("config")).map(ToString::to_string)
        };
        let version = String::from_utf8(version.stdout).ok()?;
        Some(PluginDetails {
            id: id.to_owned(),
            name,
            version,
            config: config.clone(),
            toml_str: doc,
            running: self.join_map.read().unwrap().contains_key(id),
        })
    }

    // async fn clean_loop(
    //     map: Weak<Mutex<HashMap<String, (JoinHandle<()>, JoinHandle<()>)>>>,
    //     mut rx: watch::Receiver<bool>,
    //     tx: watch::Sender<bool>,
    // ) {
    //     while rx.changed().await.is_ok() {
    //         if !*rx.borrow_and_update() {
    //             continue;
    //         }
    //         let Some(map) = map.upgrade() else {
    //             continue;
    //         };
    //         map.lock().await.retain(|_, (join_handle1, join_handle2)| {
    //             !(join_handle1.is_finished() || join_handle2.is_finished())
    //         });
    //         tx.send(false).ok();
    //     }
    // }
    pub async fn load_all(&self) -> Vec<(String, LoaderError)> {
        let mut errs = Vec::new();
        for id in self.config.keys_enabled() {
            if let Err(err) = self.load(id).await {
                errs.push((id.to_owned(), err));
            }
        }
        errs
    }

    /// # Errors
    ///
    /// # Panics
    /// - If the lock is poisoned
    pub async fn load(&self, id: &str) -> Result<bool, LoaderError> {
        let Some(config) = self.config.get(id) else {
            return Err(LoaderError::PluginConfigDoesNotExist(id.to_owned()));
        };
        if !config.enable {
            return Ok(false);
        }
        if self.join_map.read().unwrap().contains_key(id) {
            return Ok(true);
        }
        let path = std::env::current_dir()?.join("data");
        log::info!("loading [{id}]");
        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let peer = run(&config.path, &config.args)?;
        let (mut write, mut read) = split_peer(peer);
        let config_data = transport::to_value(config.config.clone())?;
        let data_path = path.join(id);
        fs::create_dir_all(&data_path)?;
        let Some(data_path) = data_path.to_str() else {
            return Err(LoaderError::PathError(format!(
                "Failed to convert data path to string for {id}"
            )));
        };
        let init_package = init_datapack(config_data, id, data_path);
        let raw = init_package.serialize_to_raw()?;
        write.send(raw).await?;
        Self::next_init_pack(&mut read).await?;
        let entry = Entry::new(Arc::downgrade(&self.join_map), id.to_owned());
        let join_handle1 = tokio::spawn(Self::write_loop(write, broadcast_rx, entry.clone()));
        let join_handle2 = tokio::spawn(Self::read_loop(read, broadcast_tx, entry));
        self.join_map
            .write()
            .unwrap()
            .insert(id.to_owned(), (join_handle1, join_handle2));
        Ok(true)
    }

    async fn next_init_pack(read: &mut FramedRead<Reader, DataPackCodec>) -> InitializeResult {
        while let Some(res) = read.next().await {
            if let Ok(res) = res {
                let matched = res.path.as_ref().map(|v| v == Initialize::<()>::path());
                if matched == Some(true) {
                    return res.payload().map_err(PluginInitError::InitPackDeserializeError)?;
                }
            }
        }
        Err(PluginInitError::ConnectionClosed)
    }

    async fn write_loop(
        mut write: FramedWrite<Writer, DataPackCodec>,
        mut broadcast_rx: broadcast::Receiver<DataPack>,
        entry: Entry,
    ) {
        while let Ok(data) = broadcast_rx.recv().await {
            if let Err(err) = write.send(data).await {
                log::log!(log::Level::Error, "Failed to send data {err}");
                if err.is_io() {
                    entry.abort();
                    return;
                }
            }
        }
    }

    async fn read_loop(
        mut read: FramedRead<Reader, DataPackCodec>,
        broadcast_tx: broadcast::Sender<DataPack>,
        entry: Entry,
    ) {
        while let Some(data) = read.next().await {
            let data = match data {
                Ok(data) => data,
                Err(err) => {
                    log::error!("Failed to read data: {err}");
                    if err.is_io() {
                        entry.abort();
                        return;
                    }
                    continue;
                }
            };
            let Some(data) = map_log(data) else {
                continue;
            };
            let result = broadcast_tx.send(data);
            if result.is_err() {
                log::error!("Failed to broadcast data");
            }
        }
    }

    // async fn clean(map: &JoinMapWeak) {
    //     let Some(map) = map.upgrade() else {
    //         return;
    //     };
    //     map.lock().await.retain(|_, (join_handle1, join_handle2)| {
    //         !(join_handle1.is_finished() || join_handle2.is_finished())
    //     });
    // }

    /// # Panics
    /// - If the lock is poisoned
    pub fn abort(&self, id: &str) {
        let Some((join_handle1, join_handle2)) = self.join_map.write().unwrap().remove(id) else {
            return;
        };
        join_handle1.abort();
        join_handle2.abort();
        log::info!("[{id}] stopped");
    }

    /// # Panics
    /// - If the lock is poisoned
    pub fn abort_all(&self) {
        for (_, (join_handle1, join_handle2)) in self.join_map.write().unwrap().drain() {
            join_handle1.abort();
            join_handle2.abort();
        }
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        // self.clean_loop.abort();
        self.abort_all();
    }
}

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Failed to start Plugin with I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("Plugin {0} config does not exist.")]
    PluginConfigDoesNotExist(String),
    #[error("Failed to parse config value: {0}")]
    ParseValueError(#[from] ValueError),
    #[error("Failed to start Plugin with Path error: {0}")]
    PathError(String),
    #[error("Failed to start Plugin with DataPack Encode error: {0}")]
    PostError(#[from] transport::EncodeError),
    #[error("Failed to start Plugin with Init error: {0}")]
    InitError(#[from] DataPackCodecError),
    #[error("{0}")]
    PluginInitError(#[from] PluginInitError),
}

fn run<P, I, S>(path: P, args: I) -> Result<Peer, io::Error>
where
    P: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(path)
        .args(args)
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    Ok(Peer::from_child(child).expect(
        "If you see this message, it means that the child process failed convert to a peer. THIS \
         IS A BUG, PLEASE REPORT IT",
    ))
}

fn split_peer(
    peer: Peer,
) -> (
    FramedWrite<Writer, DataPackCodec>,
    FramedRead<Reader, DataPackCodec>,
) {
    let (write, read) = peer.split();
    (
        FramedWrite::new(write, DataPackCodec::new()),
        FramedRead::new(read, DataPackCodec::new()),
    )
}

fn init_datapack<D1: Display, D2: Display>(
    conf: transport::Value,
    name: D1,
    data_path: D2,
) -> DataPack {
    let init = Initialize::new(conf, name, data_path);
    DataPack::builder().payload(&init).path("/initialize").build()
}

fn map_log(data: DataPack) -> Option<DataPack> {
    let is_log = data.path.as_ref().is_some_and(|v| v == "/log.create");
    if !is_log {
        return Some(data);
    }

    let Ok(payload) = data.payload::<Log>() else {
        return Some(data);
    };

    payload.log();

    None
}
