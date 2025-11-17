use crate::layed::backoff::Backoff;
use crate::layed::config::CLIENT_BACKOFF_SECS;
use crate::layed::heartbeat;
use crate::layed::magic;
use crate::tcp;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::copy_bidirectional;
use tokio::time::sleep;

static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub async fn run<Conn>(
    connect_to_gateway: impl AsyncFn() -> Result<Conn, io::Error>,
    private_addrs: &[SocketAddr],
) -> !
where
    Conn: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let mut backoff = Backoff::new(CLIENT_BACKOFF_SECS);

    loop {
        let one_round = async {
            log::info!("Connecting to gateway");
            let mut gateway = connect_to_gateway().await?;

            log::info!("Sending early handshake");
            magic::write_to(&mut gateway).await?;

            log::info!("Waiting for end of heartbeat");
            heartbeat::read_from(&mut gateway).await?;

            log::info!("Sending late handshake");
            magic::write_to(&mut gateway).await?;

            log::info!("Connecting to private");
            let mut private = tcp::connect(private_addrs).await?;

            log::info!("Spawning ({} active)", ACTIVE.fetch_add(1, Relaxed) + 1);
            tokio::spawn(async move {
                let done = copy_bidirectional(&mut gateway, &mut private).await;
                let active = ACTIVE.fetch_sub(1, Relaxed) - 1;
                match done {
                    Ok((down, up)) => log::info!("Closing ({} active): {}/{}", active, down, up),
                    Err(e) => log::info!("Closing ({} active): {}", active, e),
                }
            });

            Ok::<(), io::Error>(())
        }
        .await;

        match one_round {
            Ok(()) => {
                backoff.reset();
            }
            Err(e) => {
                log::error!("Failed: {}", e);
                let seconds = backoff.next();
                log::warn!("Retrying in {} seconds", seconds);
                sleep(Duration::from_secs(u64::from(seconds))).await;
            }
        }
    }
}
