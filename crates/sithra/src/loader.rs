use std::{env, ffi::OsStr, fmt::Display, fs, io, process::Stdio};

use ahash::HashMap;
use futures_util::{SinkExt, StreamExt};
use sithra_kit::{
    transport::{
        datapack::{DataPack, DataPackCodec},
        peer::{Peer, Reader, Writer},
    },
    types::{initialize::Initialize, log::Log},
};
use tokio::{process::Command, sync::broadcast, task::JoinHandle};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::conf::Config;

pub struct Loader {
    config:        Config,
    broadcast_tx:  broadcast::Sender<DataPack>,
    _broadcast_rx: broadcast::Receiver<DataPack>,
    join_map:      HashMap<String, (JoinHandle<()>, JoinHandle<()>)>,
}

impl Loader {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(32);
        let join_map = HashMap::default();

        Self {
            config,
            broadcast_tx,
            _broadcast_rx: broadcast_rx,
            join_map,
        }
    }

    /// # Panics
    /// If the current directory cannot be retrieved.
    pub fn load(&mut self) {
        let path = env::current_dir().expect("Failed to get current directory").join("data");
        for (name, config) in self.config.iter() {
            log::info!("Loading {name}");
            let broadcast_tx = self.broadcast_tx.clone();
            let broadcast_rx = broadcast_tx.subscribe();
            let peer = run(&config.path, &config.args);
            let peer = match peer {
                Ok(peer) => peer,
                Err(err) => {
                    log::error!("Failed to start peer {name}: {err}");
                    continue;
                }
            };
            let (write, read) = split_peer(peer);
            let config_data = rmpv::ext::to_value(config.config.clone());
            let config_data = match config_data {
                Ok(config_data) => config_data,
                Err(err) => {
                    log::error!("Failed to serialize config data for {name}: {err}");
                    continue;
                }
            };
            let data_path = path.join(name);
            if let Err(err) = fs::create_dir_all(&data_path) {
                log::error!("Failed to create data directory for {name}: {err}");
                continue;
            }
            let Some(data_path) = data_path.to_str() else {
                log::error!("Failed to convert data path to string for {name}");
                continue;
            };
            let init_package = init_datapack(config_data, data_path);
            let raw = init_package.serialize_to_raw();
            let raw = match raw {
                Ok(raw) => raw,
                Err(err) => {
                    log::error!("Failed to serialize init package for {name}: {err}");
                    continue;
                }
            };
            let join_handle1 = tokio::spawn(async move {
                let mut write = write;
                let mut broadcast_rx = broadcast_rx;

                let result = write.send(raw).await;

                if let Err(err) = result {
                    log::log!(log::Level::Error, "Failed to send init package {err}");
                    return;
                }

                while let Ok(data) = broadcast_rx.recv().await {
                    if let Err(err) = write.send(data).await {
                        log::log!(log::Level::Error, "Failed to send data {err}");
                    }
                }
            });
            let join_handle2 = tokio::spawn(async move {
                let mut read = read;
                let broadcast_tx = broadcast_tx;

                while let Some(data) = read.next().await {
                    match data {
                        Ok(data) => {
                            let Some(data) = map_log(data) else {
                                continue;
                            };
                            let result = broadcast_tx.send(data);
                            if result.is_err() {
                                log::error!("Failed to broadcast data");
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to read data: {err}");
                        }
                    }
                }
            });
            self.join_map.insert(name.to_owned(), (join_handle1, join_handle2));
        }
    }

    pub fn abort(&mut self, name: &str) {
        if let Some((join_handle1, join_handle2)) = self.join_map.remove(name) {
            join_handle1.abort();
            join_handle2.abort();
        }
    }

    pub fn abort_all(&mut self) {
        for (_, (join_handle1, join_handle2)) in self.join_map.drain() {
            join_handle1.abort();
            join_handle2.abort();
        }
    }
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

fn init_datapack<D: Display>(conf: rmpv::Value, data_path: D) -> DataPack {
    let init = Initialize::new(conf, data_path);
    DataPack::builder().payload(init).path("/initialize").build()
}

fn map_log(data: DataPack) -> Option<DataPack> {
    let is_log = data.path.as_ref().is_some_and(|v| v == "/log.create");
    if !is_log {
        return Some(data);
    }

    let Ok(payload) = data.payload::<Log>() else {
        return Some(data);
    };

    let Log {
        level,
        message,
        target,
    } = payload;

    log::log!(target: target.as_str(), level, "{message}");

    None
}
