use crate::layed::backoff::Backoff;
use crate::layed::config::CLIENT_BACKOFF_SECS;
use crate::layed::heartbeat;
use crate::layed::magic;
use crate::rw;
use crate::tcp;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run(gateway_addrs: &[SocketAddr], private_addrs: &[SocketAddr]) -> ! {
    let mut backoff = Backoff::new(CLIENT_BACKOFF_SECS);
    let active = Arc::new(AtomicUsize::new(0));

    loop {
        let one_round = async {
            log::info!("Connecting to gateway");
            let mut gateway = tcp::connect(gateway_addrs).await?;

            log::info!("Sending early handshake");
            magic::write_to(&mut gateway).await?;

            log::info!("Waiting for end of heartbeat");
            heartbeat::read_from(&mut gateway).await?;

            log::info!("Sending late handshake");
            magic::write_to(&mut gateway).await?;

            log::info!("Connecting to private");
            let private = tcp::connect(private_addrs).await?;

            log::info!("Spawning ({} active)", active.fetch_add(1, Relaxed) + 1);
            let active = active.clone();
            tokio::spawn(async move {
                let done = rw::conjoin(gateway, private).await;
                let active = active.fetch_sub(1, Relaxed) - 1;
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
