mod backoff;
mod client;
mod config;
mod err;
mod heartbeat;
mod magic;
pub mod opt;
mod rw;
mod server;
mod stream;

pub async fn main(options: opt::Options) -> Result<(), std::io::Error> {
    let opt::Options { mode } = options;

    match mode {
        opt::Mode::Server { gateway, public } => server::run(&gateway, &public).await?,
        opt::Mode::Client { gateway, private } => client::run(&gateway, &private).await,
    }

    Ok(())
}
