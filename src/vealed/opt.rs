use crate::opt::SocketAddrsFromDns;
use clap::Args;
use std::net::SocketAddr;

/// Forward TCP connections somewhere else
#[derive(Args, Debug)]
pub struct Options {
    /// Socket address to listen on
    pub listen: SocketAddr,

    /// Address to forward connections to
    #[arg(short, long)]
    pub to: SocketAddrsFromDns,
}
