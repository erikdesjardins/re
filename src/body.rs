use futures::TryStreamExt;
use http_body_util::StreamBody;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Bytes, Frame};
use std::io;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub fn empty<E>() -> BoxBody<Bytes, E> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub fn from_file(file: File) -> impl Body<Data = Bytes, Error = io::Error> {
    let stream = ReaderStream::with_capacity(file, 64 * 1024);
    StreamBody::new(stream.map_ok(Frame::data))
}
