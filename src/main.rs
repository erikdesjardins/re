#![allow(clippy::type_complexity, clippy::manual_map)]

#[macro_use]
mod macros;

mod directed;
mod flected;
mod layed;
mod transmitted;
mod vealed;

mod body;
mod config;
mod err;
mod future;
mod http;
mod opt;
mod rw;
mod tcp;

#[tokio::main]
async fn main() -> Result<(), err::DisplayError> {
    let opt::Options { verbose, command } = clap::Parser::parse();

    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    match command {
        opt::Command::Directed(options) => directed::main(options).await?,
        opt::Command::Flected(options) => flected::main(options).await?,
        opt::Command::Layed(options) => layed::main(options).await?,
        opt::Command::Transmitted(options) => transmitted::main(options).await?,
        opt::Command::Vealed(options) => vealed::main(options).await?,
    }

    Ok(())
}
