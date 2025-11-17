use bytes::{Buf, Bytes};
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use http::Uri;
use std::cmp;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{accept_async_with_config, connect_async_with_config};

use crate::tcp;

pub async fn connect(
    url: &Uri,
    buffer_size: usize,
) -> Result<impl AsyncRead + AsyncWrite + use<>, io::Error> {
    let config = WebSocketConfig::default().max_message_size(Some(buffer_size));
    let set_nodelay = true;
    let (stream, _) = connect_async_with_config(url, Some(config), set_nodelay)
        .await
        .map_err(io::Error::other)?;

    Ok(WebSocketStream {
        stream,
        current_read_frame: None,
        current_write_frame: None,
        last_written_frame: None,
        buffer_size,
    })
}

pub async fn accept(
    listener: &mut TcpListener,
    buffer_size: usize,
) -> Result<impl AsyncRead + AsyncWrite + use<>, io::Error> {
    let stream = tcp::accept(listener).await?;

    let config = WebSocketConfig::default().max_message_size(Some(buffer_size));
    let stream = accept_async_with_config(stream, Some(config))
        .await
        .map_err(io::Error::other)?;

    Ok(WebSocketStream {
        stream,
        current_read_frame: None,
        current_write_frame: None,
        last_written_frame: None,
        buffer_size,
    })
}

struct WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream: tokio_tungstenite::WebSocketStream<S>,
    current_read_frame: Option<Bytes>,
    current_write_frame: Option<BytesMut>,
    last_written_frame: Option<Bytes>,
    buffer_size: usize,
}

impl<S> AsyncRead for WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Read next frame from the websocket if we don't have any data buffered
        let frame = match &mut self.current_read_frame {
            Some(frame) if !frame.is_empty() => frame,
            _ => match self.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(Message::Binary(bytes)))) => {
                    self.current_read_frame.insert(bytes)
                }
                Poll::Ready(Some(Ok(Message::Close(_)))) => {
                    return Poll::Ready(Ok(())); // EOF
                }
                Poll::Ready(Some(Ok(_))) => {
                    return Poll::Ready(Err(io::Error::other("unexpected websocket message type")));
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::other(e)));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(Ok(())); // EOF
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            },
        };

        // Copy data from the current frame into the provided buffer
        let to_read = cmp::min(buf.remaining(), frame.len());
        buf.put_slice(&frame[..to_read]);
        frame.advance(to_read);

        Poll::Ready(Ok(()))
    }
}

impl<S> WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_shallow_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match &mut self.current_write_frame {
            Some(frame) if !frame.is_empty() => match self.stream.poll_ready_unpin(cx) {
                Poll::Ready(Ok(())) => {
                    let frame = self.current_write_frame.take().unwrap().freeze();
                    self.last_written_frame = Some(frame.clone());

                    match self.stream.start_send_unpin(Message::Binary(frame)) {
                        Ok(()) => Poll::Ready(Ok(())),
                        Err(e) => Poll::Ready(Err(io::Error::other(e))),
                    }
                }
                Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::other(e))),
                Poll::Pending => Poll::Pending,
            },
            // No pending data to write
            _ => Poll::Ready(Ok(())),
        }
    }
}

impl<S> AsyncWrite for WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        // If the current frame is full, flush it first
        if let Some(frame) = &self.current_write_frame
            && frame.len() == frame.capacity()
        {
            match self.poll_shallow_flush(cx) {
                Poll::Ready(Ok(())) => {
                    // Continue (flushed successfully)
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        // Ensure that we have a frame to write into
        let frame = match &mut self.current_write_frame {
            Some(frame) => frame,
            None => match self.last_written_frame.take() {
                // If we can reclaim the last written frame without cloning, do so
                Some(last_frame) if last_frame.is_unique() => {
                    let mut new_frame = BytesMut::from(last_frame);
                    new_frame.clear();
                    self.current_write_frame.insert(new_frame)
                }
                // Otherwise, allocate a new frame
                _ => {
                    let new_frame = BytesMut::with_capacity(self.buffer_size);
                    self.current_write_frame.insert(new_frame)
                }
            },
        };

        // Write data into the current frame
        let to_write = cmp::min(buf.len(), frame.capacity() - frame.len());
        frame.put_slice(&buf[..to_write]);

        Poll::Ready(Ok(to_write))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // Write any pending data in our buffers as a websocket frame
        match self.poll_shallow_flush(cx) {
            Poll::Ready(Ok(())) => {
                // Continue (flushed successfully)
            }
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }

        // Flush the websocket stream
        match self.stream.poll_flush_unpin(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        // Flush data first, as required by AsyncWrite contract
        match self.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => {
                // Continue (flushed successfully)
            }
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }

        // Then close the websocket
        match self.stream.poll_close_unpin(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}
