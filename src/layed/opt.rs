use crate::opt::SocketAddrsFromDns;
use clap::{Args, Subcommand};
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
    },
    /// Run the client half on a private machine
    Client {
        /// Address of server's gateway
        gateway: SocketAddrsFromDns,

        /// Address to relay public traffic to
        private: SocketAddrsFromDns,
    },
}
