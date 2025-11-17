use crate::flected::body::BytesBody;
use bytes::Bytes;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tokio::sync::RwLock;

mod index;
mod paths;

#[derive(Default)]
pub struct State {
    files: RwLock<BTreeMap<String, Bytes>>,
}

pub async fn respond_to_request(req: Request<Incoming>, state: &State) -> Response<BytesBody> {
    match *req.method() {
        Method::GET if req.uri().path() == "/" => index::get(req, state).await,
        Method::GET => paths::get(req, state).await,
        Method::POST => paths::post(req, state).await,
        Method::DELETE => paths::delete(req, state).await,
        _ => {
            log::warn!("{} {} -> [method not allowed]", req.method(), req.uri());
            let mut resp = Response::new(BytesBody::empty());
            *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            resp
        }
    }
}
