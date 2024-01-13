use crate::err::{AppliesTo, IoErrorExt};
use crate::layed::backoff::Backoff;
use crate::layed::config::{QUEUE_TIMEOUT, SERVER_ACCEPT_BACKOFF_SECS};
use crate::layed::heartbeat;
use crate::layed::magic;
use crate::layed::stream::spawn_idle;
use crate::rw;
use crate::tcp;
use futures::future::{select, Either};
use futures::stream;
use futures::StreamExt;
use std::io;
use std::net::SocketAddr;
use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::error::Elapsed;
use tokio::time::{sleep, timeout};

static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub async fn run(gateway_addr: &SocketAddr, public_addr: &SocketAddr) -> Result<(), io::Error> {
    log::info!("Binding to gateway: {}", gateway_addr);
    let gateway_connections = TcpListener::bind(gateway_addr).await?;
    log::info!("Binding to public: {}", public_addr);
    let mut public_connections = TcpListener::bind(public_addr).await?;

    let mut gateway_connections = pin!(spawn_idle(|requests| {
        stream::unfold(
            (gateway_connections, requests),
            |(mut gateway_connections, mut requests)| async {
                loop {
                    let mut backoff = Backoff::new(SERVER_ACCEPT_BACKOFF_SECS);
                    let mut gateway = loop {
                        match tcp::accept(&mut gateway_connections).await {
                            Ok(gateway) => break gateway,
                            Err(e) => {
                                log::error!("Error accepting gateway connections: {}", e);
                                let seconds = backoff.next();
                                log::warn!("Retrying in {} seconds", seconds);
                                sleep(Duration::from_secs(u64::from(seconds))).await;
                                continue;
                            }
                        }
                    };

                    // early handshake: immediately kill unknown connections
                    match magic::read_from(&mut gateway).await {
                        Ok(()) => log::info!("Early handshake succeeded"),
                        Err(e) => {
                            log::info!("Early handshake failed: {}", e);
                            continue;
                        }
                    }

                    // heartbeat: so the client can tell if the connection drops
                    let token = {
                        let heartbeat = pin!(heartbeat::write_forever(&mut gateway));
                        match select(requests.next(), heartbeat).await {
                            Either::Left((Some(token), _)) => token,
                            Either::Left((None, _)) => return None,
                            Either::Right((Ok(i), _)) => match i {},
                            Either::Right((Err(e), _)) => {
                                log::info!("Heartbeat failed: {}", e);
                                continue;
                            }
                        }
                    };

                    return Some(((token, gateway), (gateway_connections, requests)));
                }
            },
        )
    }));

    'public: loop {
        let mut backoff = Backoff::new(SERVER_ACCEPT_BACKOFF_SECS);
        let public = loop {
            match tcp::accept(&mut public_connections).await {
                Ok(public) => break public,
                Err(e) => {
                    log::error!("Error accepting public connections: {}", e);
                    let seconds = backoff.next();
                    log::warn!("Retrying in {} seconds", seconds);
                    sleep(Duration::from_secs(u64::from(seconds))).await;
                    continue;
                }
            }
        };

        let gateway = loop {
            // drop public connections which wait for too long, to avoid unlimited queuing when no gateway is connected
            let mut gateway = match timeout(QUEUE_TIMEOUT, gateway_connections.next()).await {
                Ok(Some(gateway)) => gateway,
                Ok(None) => return Ok(()),
                Err(e) => {
                    let _: Elapsed = e;
                    log::info!("Public connection expired waiting for gateway");
                    drain_queue(&mut public_connections).await;
                    continue 'public;
                }
            };

            // finish heartbeat: do this as late as possible so clients can't send late handshake and disconnect
            match heartbeat::write_final(&mut gateway).await {
                Ok(()) => log::info!("Heartbeat completed"),
                Err(e) => {
                    log::info!("Heartbeat failed at finalization: {}", e);
                    continue;
                }
            }

            // late handshake: ensure that client hasn't disappeared some time after early handshake
            match magic::read_from(&mut gateway).await {
                Ok(()) => log::info!("Late handshake succeeded"),
                Err(e) => {
                    log::info!("Late handshake failed: {}", e);
                    continue;
                }
            }

            break gateway;
        };

        log::info!("Spawning ({} active)", ACTIVE.fetch_add(1, Relaxed) + 1);
        tokio::spawn(async move {
            let done = rw::conjoin(public, gateway).await;
            let active = ACTIVE.fetch_sub(1, Relaxed) - 1;
            match done {
                Ok((down, up)) => log::info!("Closing ({} active): {}/{}", active, down, up),
                Err(e) => log::info!("Closing ({} active): {}", active, e),
            }
        });
    }
}

pub async fn drain_queue(listener: &mut TcpListener) {
    loop {
        // timeout because we need to yield to receive the second queued conn
        // (listener.poll_recv() won't return Poll::Ready twice in a row,
        //  even if there are multiple queued connections)
        match timeout(Duration::from_millis(1), listener.accept()).await {
            Ok(Ok((_, _))) => log::info!("Queued conn dropped"),
            Ok(Err(e)) => match e.applies_to() {
                AppliesTo::Connection => log::info!("Queued conn dropped: {}", e),
                AppliesTo::Listener => break,
            },
            Err(e) => {
                let _: Elapsed = e;
                break;
            }
        }
    }
}
