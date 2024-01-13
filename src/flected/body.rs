use crate::flected::as_ref::{ForwardAsRef, ReindexAsRef};
use hyper::body::{Body, Frame, SizeHint};
use std::convert::Infallible;
use std::io::Cursor;
use std::mem;
use std::ops::Range;
use std::sync::Arc;
use std::task::Context;
use tokio::macros::support::{Pin, Poll};

type ArcAsRefBytes = Arc<dyn AsRef<[u8]> + Sync + Send>;

pub struct ArcBody {
    data: Option<ArcAsRefBytes>,
    range: Range<usize>,
}

impl ArcBody {
    pub fn new<T>(bytes: T) -> Self
    where
        T: AsRef<[u8]> + Sync + Send + 'static,
    {
        Self::from_arc(Arc::new(bytes))
    }

    pub fn from_arc<T>(arc: Arc<T>) -> Self
    where
        T: AsRef<[u8]> + Sync + Send + 'static,
    {
        Self {
            range: 0..T::as_ref(&arc).len(),
            data: Some(arc),
        }
    }

    pub fn from_arc_with_range<T>(arc: Arc<T>, range: Range<usize>) -> Result<Self, Arc<T>>
    where
        T: AsRef<[u8]> + Sync + Send + 'static,
    {
        // check if the range is in bounds
        match T::as_ref(&arc).get(range.clone()) {
            Some(_) => Ok(Self {
                data: Some(arc),
                range,
            }),
            None => Err(arc),
        }
    }

    pub fn empty() -> Self {
        Self {
            data: None,
            range: 0..0,
        }
    }
}

impl Body for ArcBody {
    type Data = Cursor<ReindexAsRef<ForwardAsRef<ArcAsRefBytes>, Range<usize>>>;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let Self { data, range } = &mut *self;

        // windows/linux can't handle write calls bigger than this
        let chunk_size = i32::MAX as usize;

        let (data, range) = match data {
            Some(data) if (range.end - range.start) > chunk_size => {
                let split = range.start + chunk_size;
                let (first, rest) = (range.start..split, split..range.end);
                *range = rest;
                (Arc::clone(data), first)
            }
            data @ Some(_) => {
                // can send everything in one shot
                (data.take().unwrap(), mem::replace(range, 0..0))
            }
            None => return Poll::Ready(None),
        };

        Poll::Ready(Some(Ok(Frame::data(Cursor::new(ReindexAsRef(
            ForwardAsRef(data),
            range,
        ))))))
    }

    fn is_end_stream(&self) -> bool {
        self.range.start == self.range.end
    }

    fn size_hint(&self) -> SizeHint {
        let len = self.range.end - self.range.start;
        SizeHint::with_exact(len as u64)
    }
}
