use std::{
    env::{self, VarError},
    net::{AddrParseError, IpAddr},
    num::ParseIntError,
};

use thiserror::Error;

pub fn host() -> Result<IpAddr, GetAddrError> {
    let host_str = env::var("SITHRA_WEB_HOST").map_err(GetAddrError::HostVar)?;
    Ok(host_str.parse()?)
}

pub fn port() -> Result<u16, GetAddrError> {
    let port_str = env::var("SITHRA_WEB_PORT").map_err(GetAddrError::PortVar)?;
    Ok(port_str.parse()?)
}

pub fn addr(default: (IpAddr, u16)) -> (IpAddr, u16) {
    let (default_host, default_port) = default;
    let host = match host() {
        Ok(host) => host,
        Err(err) => {
            log::warn!("{err}, using localhost");
            default_host
        }
    };
    let port = match port() {
        Ok(port) => port,
        Err(err) => {
            log::warn!("{err}, using default port 8080");
            default_port
        }
    };
    (host, port)
}

pub fn addr_display(addr: (IpAddr, u16)) -> String {
    let (host, port) = addr;
    match host {
        IpAddr::V4(ip) => format!("{ip}:{port}"),
        IpAddr::V6(ip) => format!("[{ip}]:{port}"),
    }
}

#[derive(Debug, Error)]
pub enum GetAddrError {
    #[error("Failed to parse host: {0}")]
    ParseHost(#[from] AddrParseError),
    #[error("Failed to get host from environment: {0}")]
    HostVar(VarError),
    #[error("Failed to parse port: {0}")]
    ParsePort(#[from] ParseIntError),
    #[error("Failed to get port from environment: {0}")]
    PortVar(VarError),
}
