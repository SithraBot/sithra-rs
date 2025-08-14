use std::net::IpAddr;

use clap::Parser;

/// Defines command-line arguments and environment variables for server
/// configuration using clap.
///
/// Clap automatically handles the priority:
/// 1. Command-line arguments (e.g., `--host 127.0.0.1`)
/// 2. Environment variables (e.g., `SITHRA_WEB_HOST=127.0.0.1`)
#[derive(Parser, Debug)]
pub struct Args {
    /// The IP address for the server to bind to.
    #[arg(long, env = "SITHRA_WEB_HOST")]
    host: Option<IpAddr>,

    /// The port for the server to bind to.
    #[arg(long, env = "SITHRA_WEB_PORT")]
    port: Option<u16>,

    /// The path for the web server to serve files from.
    #[arg(long, env = "SITHRA_WEB_PATH")]
    web_path: Option<String>,

    /// Whether to run the server in API-only mode.
    #[arg(long, env = "SITHRA_WEB_API_ONLY")]
    api_only: bool,
}

impl Args {
    /// Determines the server's bind address (host and port) using command-line
    /// arguments, environment variables, or provided defaults.
    ///
    /// The address is determined with the following priority:
    /// 1. Command-line arguments (`--host`, `--port`)
    /// 2. Environment variables (`SITHRA_WEB_HOST`, `SITHRA_WEB_PORT`)
    /// 3. The provided default values.
    ///
    /// If a value is not provided by arguments or environment variables, a
    /// warning is logged and the corresponding default value is used.
    ///
    /// # Arguments
    ///
    /// * `default` - A tuple `(IpAddr, u16)` containing the default host and
    ///   port.
    pub fn addr(&self, default: (IpAddr, u16)) -> (IpAddr, u16) {
        let (default_host, default_port) = default;

        let host = if let Some(host) = self.host {
            host
        } else {
            log::warn!(
                "Host not specified via --host or SITHRA_WEB_HOST, using default: {default_host}",
            );
            default_host
        };

        let port = if let Some(port) = self.port {
            port
        } else {
            log::warn!(
                "Port not specified via --port or SITHRA_WEB_PORT, using default: {default_port}",
            );
            default_port
        };

        (host, port)
    }

    /// Returns the web path, or a default value if not specified.
    ///
    /// If a value is not provided by arguments or environment variables, a
    /// warning is logged and the corresponding default value is used.
    ///
    /// # Arguments
    ///
    /// * `default` - A string containing the default web path.
    pub fn web_path(&self, default: &str) -> String {
        if let Some(ref path) = self.web_path {
            path
        } else {
            log::warn!(
                "Web path not specified via --web-path or SITHRA_WEB_PATH, using default: \
                 {default}",
            );
            default
        }
        .to_owned()
    }

    /// Returns whether the server should run in API-only mode.
    pub const fn api_only(&self) -> bool {
        self.api_only
    }
}

/// Formats an IP address and port into a displayable string.
///
/// Handles IPv6 addresses by wrapping them in square brackets.
pub fn addr_display(addr: (IpAddr, u16)) -> String {
    let (host, port) = addr;
    match host {
        IpAddr::V4(ip) => format!("{ip}:{port}"),
        IpAddr::V6(ip) => format!("[{ip}]:{port}"),
    }
}
