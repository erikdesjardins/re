use clap::{ArgAction, Parser, Subcommand};

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
