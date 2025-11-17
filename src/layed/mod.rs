use crate::tcp;
use crate::websocket;

mod backoff;
mod client;
mod config;
mod heartbeat;
mod magic;
pub mod opt;
mod server;
mod stream;

pub async fn main(options: opt::Options) -> Result<(), std::io::Error> {
    let opt::Options { mode } = options;

    match mode {
        opt::Mode::Server {
            gateway,
            public,
            websocket,
        } => {
            if websocket {
                server::run(&gateway, &public, async |mut listener| {
                    (websocket::accept(&mut listener).await, listener)
                })
                .await?;
            } else {
                server::run(&gateway, &public, async |mut listener| {
                    (tcp::accept(&mut listener).await, listener)
                })
                .await?;
            }
        }
        opt::Mode::Client {
            gateway,
            private,
            websocket,
        } => match websocket {
            opt::WebSocketEnabled::Insecure | opt::WebSocketEnabled::Secure => {
                let scheme = if websocket == opt::WebSocketEnabled::Insecure {
                    "ws"
                } else {
                    "wss"
                };
                let uri = http::uri::Builder::new()
                    .scheme(scheme)
                    .authority(gateway.orig())
                    .path_and_query("/ws/")
                    .build()
                    .unwrap();
                client::run(|| websocket::connect(&uri), &private).await;
            }
            opt::WebSocketEnabled::Off => {
                client::run(|| tcp::connect(&gateway), &private).await;
            }
        },
    }

    Ok(())
}
