use clap::Args;
use std::net::SocketAddr;

/// Simple CORS proxy
#[derive(Args, Debug)]
pub struct Options {
    /// Socket address to listen on
    pub listen: SocketAddr,

    // TODO: use a group to forbid setting both of these when https://github.com/clap-rs/clap/pull/4688 is merged
    /// Secret key that clients must provide in the x-retransmitted-key header
    #[arg(long, required = false)]
    pub secret_key: String,

    /// Allow unauthenticated access with no secret key
    #[arg(long)]
    pub no_secret_key: bool,
}
