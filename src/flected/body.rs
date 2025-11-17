use bytes::Bytes;
use hyper::body::{Body, Frame, SizeHint};
use std::task::Context;
use std::{cmp, convert::Infallible};
use tokio::macros::support::{Pin, Poll};

pub struct BytesBody(Bytes);

impl BytesBody {
    pub fn new(bytes: Bytes) -> Self {
        Self(bytes)
    }

    pub fn empty() -> Self {
        Self(Bytes::new())
    }
}

impl From<String> for BytesBody {
    fn from(s: String) -> Self {
        Self(Bytes::from(s))
    }
}

impl Body for BytesBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if self.0.is_empty() {
            return Poll::Ready(None);
        }

        // windows/linux can't handle write calls bigger than this
        let chunk_size = i32::MAX as usize;
        let bytes_to_read = cmp::min(self.0.len(), chunk_size);
        let read = self.0.split_to(bytes_to_read);

        Poll::Ready(Some(Ok(Frame::data(read))))
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_empty()
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(u64::try_from(self.0.len()).unwrap())
    }
}
