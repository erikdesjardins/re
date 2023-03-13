use clap::Args;
use std::net::SocketAddr;

/// Temporarily upload and serve files from memory
#[derive(Args, Debug)]
pub struct Options {
    /// Socket address to listen on
    pub listen: SocketAddr,
}
