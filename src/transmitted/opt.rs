use clap::Args;
use std::net::SocketAddr;

/// Simple CORS proxy
#[derive(Args, Debug)]
pub struct Options {
    /// Socket address to listen on
    pub listen: SocketAddr,

    #[command(flatten)]
    pub key: KeyOptions,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct KeyOptions {
    /// Secret key that clients must provide in the x-retransmitted-key header
    #[arg(long)]
    pub secret_key: Option<String>,

    /// Allow unauthenticated access with no secret key
    #[arg(long)]
    pub no_secret_key: bool,
}
