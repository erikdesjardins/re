use crate::err::{AppliesTo, IoErrorExt};
use crate::future::first_ok;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub async fn connect(addrs: &[SocketAddr]) -> Result<TcpStream, io::Error> {
    let stream = first_ok(addrs.iter().map(TcpStream::connect)).await?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub async fn accept(listener: &mut TcpListener) -> Result<TcpStream, io::Error> {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                stream.set_nodelay(true)?;
                return Ok(stream);
            }
            Err(e) => match e.applies_to() {
                AppliesTo::Connection => log::debug!("Aborted connection dropped: {}", e),
                AppliesTo::Listener => return Err(e),
            },
        }
    }
}
