use crate::config::COPY_BUFFER_SIZE;
use crate::tcp;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use tokio::io::copy_bidirectional_with_sizes;
use tokio::net::TcpListener;

static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub async fn run(from_addr: SocketAddr, to_addrs: &[SocketAddr]) -> Result<(), io::Error> {
    log::info!("Binding to: {}", from_addr);
    let mut connections = TcpListener::bind(from_addr).await?;

    loop {
        let mut inbound = tcp::accept(&mut connections).await?;

        let mut outbound = match tcp::connect(to_addrs).await {
            Ok(outbound) => outbound,
            Err(e) => {
                log::error!("Failed to connect: {}", e);
                continue;
            }
        };

        log::info!("Spawning ({} active)", ACTIVE.fetch_add(1, Relaxed) + 1);
        tokio::spawn(async move {
            let done = copy_bidirectional_with_sizes(
                &mut inbound,
                &mut outbound,
                COPY_BUFFER_SIZE,
                COPY_BUFFER_SIZE,
            )
            .await;
            let active = ACTIVE.fetch_sub(1, Relaxed) - 1;
            match done {
                Ok((down, up)) => log::info!("Closing ({} active): {}/{}", active, down, up),
                Err(e) => log::info!("Closing ({} active): {}", active, e),
            }
        });
    }
}
