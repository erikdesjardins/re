mod backoff;
mod client;
mod config;
mod err;
mod future;
mod heartbeat;
mod magic;
pub mod opt;
mod rw;
mod server;
mod stream;

pub async fn main(options: opt::Options) -> Result<(), std::io::Error> {
    let opt::Options { mode } = options;

    let local = tokio::task::LocalSet::new();

    match mode {
        opt::Mode::Server { gateway, public } => {
            local
                .run_until(server::run(&local, &gateway, &public))
                .await?;
        }
        opt::Mode::Client { gateway, private } => {
            local
                .run_until(client::run(&local, &gateway, &private))
                .await;
        }
    }

    Ok(())
}
