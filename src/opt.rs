use clap::{ArgAction, Parser, Subcommand};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Options {
    /// Logging verbosity (-v debug, -vv trace)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Directed(crate::directed::opt::Options),
    Flected(crate::flected::opt::Options),
    Layed(crate::layed::opt::Options),
    Transmitted(crate::transmitted::opt::Options),
    Vealed(crate::vealed::opt::Options),
}

#[derive(Clone, Debug)]
pub struct SocketAddrsFromDns {
    orig: String,
    addrs: Vec<SocketAddr>,
}

impl SocketAddrsFromDns {
    pub fn orig(&self) -> &str {
        &self.orig
    }

    pub fn addrs(&self) -> &[SocketAddr] {
        &self.addrs
    }
}

impl Deref for SocketAddrsFromDns {
    type Target = [SocketAddr];

    fn deref(&self) -> &Self::Target {
        self.addrs()
    }
}

impl FromStr for SocketAddrsFromDns {
    type Err = io::Error;

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        let addrs = arg.to_socket_addrs()?.collect::<Vec<_>>();
        match addrs.len() {
            0 => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "Resolved to zero addresses",
            )),
            _ => Ok(Self {
                orig: arg.to_string(),
                addrs,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Options::command().debug_assert();
    }
}
