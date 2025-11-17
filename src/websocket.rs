use http::Uri;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async_with_config, connect_async_with_config};

use crate::tcp;

pub async fn connect(url: &Uri) -> Result<impl AsyncRead + AsyncWrite + use<>, io::Error> {
    let set_nodelay = true;
    let (stream, _) = connect_async_with_config(url, None, set_nodelay)
        .await
        .map_err(io::Error::other)?;

    Ok(stream.into_inner())
}

pub async fn accept(
    listener: &mut TcpListener,
) -> Result<impl AsyncRead + AsyncWrite + use<>, io::Error> {
    let stream = tcp::accept(listener).await?;

    let stream = accept_async_with_config(stream, None)
        .await
        .map_err(io::Error::other)?;

    Ok(stream.into_inner())
}
