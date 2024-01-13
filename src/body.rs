use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;

pub fn empty<E>() -> BoxBody<Bytes, E> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
