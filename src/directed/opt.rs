use crate::directed::redir::{From, To};
use clap::Args;
use std::net::SocketAddr;
use std::str::FromStr;

/// Redirect local HTTP traffic somewhere else
#[derive(Args, Debug)]
pub struct Options {
    #[arg(
        help = "Socket address to listen on (--help for more)",
        long_help = r"Socket address to listen on:
    - incoming http connections are received on this socket
Examples:
    - 127.0.0.1:3000
    - 0.0.0.0:80
    - [2001:db8::1]:8080"
    )]
    pub listen: SocketAddr,

    #[arg(
        help = "Path prefixes to redirect from (--help for more)",
        long_help = r"Path prefixes to redirect from:
    - each prefix is checked in order, and the first match is chosen
    - 404s if no prefixes match
Examples:
    - /
    - /resources/static/"
    )]
    #[arg(short, long, required = true, display_order = 0, value_parser = From::from_str)]
    pub from: Vec<From>,

    #[arg(
        help = "Address prefixes to redirect to (--help for more)",
        long_help = r"Address prefixes to redirect to:
    - each matching request's tail is appended to the corresponding address prefix
    - some schemes have special behavior
Examples:
    - http://localhost:8080/services/api/
    - https://test.dev/v1/
    - file://./static/
    - file://./static/|./static/index.html (fallback to ./static/index.html)
    - status://404 (empty response with status 404)"
    )]
    #[arg(short, long, required = true, display_order = 0, value_parser = To::from_str)]
    pub to: Vec<To>,
}
