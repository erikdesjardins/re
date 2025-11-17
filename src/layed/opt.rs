use crate::opt::SocketAddrsFromDns;
use clap::{Args, Subcommand, ValueEnum};
use std::net::SocketAddr;

/// Relay TCP connections to a machine behind a dynamic IP/firewall
#[derive(Args, Debug)]
pub struct Options {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Run the server half on a public machine
    Server {
        /// Socket address to receive gateway connections from client
        gateway: SocketAddr,

        /// Socket address to receive public traffic on
        public: SocketAddr,

        /// Whether to use a WebSocket instead of raw TCP for the gateway.
        ///
        /// This has worse performance, but allows traversal of HTTP-only proxies.
        /// If used, the client must also enable this option.
        #[arg(long)]
        websocket: bool,
    },
    /// Run the client half on a private machine
    Client {
        /// Address of server's gateway
        gateway: SocketAddrsFromDns,

        /// Address to relay public traffic to
        private: SocketAddrsFromDns,

        /// Whether to use a WebSocket instead of raw TCP for the gateway.
        ///
        /// This has worse performance, but allows traversal of HTTP-only proxies.
        /// If used, the server must also enable this option.
        #[arg(long, value_enum, default_value_t = WebSocketEnabled::Off)]
        websocket: WebSocketEnabled,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq)]
pub enum WebSocketEnabled {
    Off,
    Insecure,
    Secure,
}
