use clap::Args;
use std::net::SocketAddr;

#[derive(Args, Debug)]
#[clap(
    about = "Simple CORS proxy",
    long_about = "Simple CORS proxy

Clients should provide the target URL in the request path.
Examples:
- GET http://localhost:8080/http://example.com/test.html
- POST http://localhost:8080/example.com/test.html (defaults to https)"
)]
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
