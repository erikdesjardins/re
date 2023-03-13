use clap::Args;
use std::net::SocketAddr;

/// Simple CORS proxy
#[derive(Args, Debug)]
pub struct Options {
    /// Socket address to listen on
    pub listen: SocketAddr,

    /// Secret key, clients must provide it in the x-retransmitted-key header
    pub secret_key: String,
}
