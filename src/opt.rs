use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Options {
    /// Logging verbosity (-v info, -vv debug, -vvv trace)
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
}
