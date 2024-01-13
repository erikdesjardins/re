use futures::stream;
use http_body_util::StreamBody;
use hyper::body::{Body, Bytes, Frame};
use std::io;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub fn body_stream(file: File) -> impl Body<Data = Bytes, Error = io::Error> {
    StreamBody::new(stream::try_unfold(file, {
        let mut buf = [0; 4 * 1024];
        move |mut file| async move {
            match file.read(&mut buf).await {
                Ok(0) => Ok(None),
                Ok(n) => Ok(Some((Frame::data(Bytes::copy_from_slice(&buf[..n])), file))),
                Err(e) => Err(e),
            }
        }
    }))
}
