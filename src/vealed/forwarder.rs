use crate::rw;
use crate::tcp;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run(from_addr: SocketAddr, to_addrs: &[SocketAddr]) -> Result<(), io::Error> {
    let active = Arc::new(AtomicUsize::new(0));

    log::info!("Binding to: {}", from_addr);
    let mut connections = TcpListener::bind(from_addr).await?;

    loop {
        let inbound = tcp::accept(&mut connections).await?;

        let outbound = match tcp::connect(to_addrs).await {
            Ok(outbound) => outbound,
            Err(e) => {
                log::error!("Failed to connect: {}", e);
                continue;
            }
        };

        log::info!("Spawning ({} active)", active.fetch_add(1, Relaxed) + 1);
        let active = active.clone();
        tokio::spawn(async move {
            let done = rw::conjoin(inbound, outbound).await;
            let active = active.fetch_sub(1, Relaxed) - 1;
            match done {
                Ok((down, up)) => log::info!("Closing ({} active): {}/{}", active, down, up),
                Err(e) => log::info!("Closing ({} active): {}", active, e),
            }
        });
    }
}
