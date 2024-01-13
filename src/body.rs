use futures::stream;
use http_body_util::combinators::BoxBody;
use http_body_util::StreamBody;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Bytes, Frame};
use std::io;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub fn empty<E>() -> BoxBody<Bytes, E> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub fn from_file(file: File) -> impl Body<Data = Bytes, Error = io::Error> {
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
