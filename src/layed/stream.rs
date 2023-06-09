use futures::stream;
use futures::{Stream, StreamExt};
use std::mem::ManuallyDrop;
use std::pin::{pin, Pin};
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;

/// Spawns a stream onto the local set to perform idle work.
/// This keeps polling the inner stream even when no item is demanded by the parent,
/// allowing it to keep making progress.
pub fn spawn_idle<T, S>(f: impl FnOnce(Requests) -> S) -> impl Stream<Item = T>
where
    T: Send + 'static,
    S: Stream<Item = (RequestToken, T)> + Send + 'static,
{
    let (request, requests) = mpsc::channel(1);
    let (response, responses) = mpsc::channel(1);

    let idle = f(Requests(requests));
    tokio::spawn(async move {
        let mut idle = pin!(idle);
        loop {
            match idle.next().await {
                Some((token, val)) => match response.send((ManuallyDrop::new(token), val)).await {
                    Ok(()) => continue,
                    Err(mpsc::error::SendError(_)) => return,
                },
                None => return,
            }
        }
    });

    stream::unfold(
        (request, responses, ManuallyDrop::new(RequestToken(()))),
        |(request, mut responses, token)| async {
            match request.send(token).await {
                Ok(()) => match responses.recv().await {
                    Some((token, val)) => Some((val, (request, responses, token))),
                    None => None,
                },
                Err(mpsc::error::SendError(_)) => None,
            }
        },
    )
}

pub struct RequestToken(());

impl Drop for RequestToken {
    fn drop(&mut self) {
        panic!("Deadlock: request token dropped");
    }
}

pub struct Requests(mpsc::Receiver<ManuallyDrop<RequestToken>>);

impl Stream for Requests {
    type Item = RequestToken;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0
            .poll_recv(cx)
            .map(|p| p.map(ManuallyDrop::into_inner))
    }
}
