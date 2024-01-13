use crate::body;
use crate::http::ProxyClient;
use crate::transmitted::path::extract_uri_from_path;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::http::HeaderValue;
use hyper::{header, Method, Request, Response, StatusCode};
use sha2::{Digest, Sha256};
use std::mem;

pub struct State {
    pub client: ProxyClient,
    pub secret_key_hash: Option<Box<[u8]>>,
}

#[allow(clippy::declare_interior_mutable_const)]
pub async fn respond_to_request(
    mut req: Request<Incoming>,
    state: &State,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    const X_RETRANSMITTED_KEY: &str = "x-retransmitted-key";
    const ANY: HeaderValue = HeaderValue::from_static("*");
    const ALLOWED_HEADERS: HeaderValue = HeaderValue::from_static(X_RETRANSMITTED_KEY);

    if req.method() == Method::OPTIONS {
        log::info!("{} {} -> [preflight response]", req.method(), req.uri());
        let mut resp = Response::new(body::empty());
        resp.headers_mut()
            .append(header::ACCESS_CONTROL_ALLOW_ORIGIN, ANY);
        resp.headers_mut()
            .append(header::ACCESS_CONTROL_ALLOW_HEADERS, ALLOWED_HEADERS);
        return resp;
    }

    if let Some(secret_key_hash) = &state.secret_key_hash {
        let provided_key = match req.headers_mut().remove(X_RETRANSMITTED_KEY) {
            Some(k) => k,
            None => {
                log::info!("{} {} -> [missing key]", req.method(), req.uri());
                let mut resp = Response::new(body::empty());
                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                return resp;
            }
        };
        let provided_key_hash = Sha256::digest(provided_key);
        match ring::constant_time::verify_slices_are_equal(
            provided_key_hash.as_slice(),
            secret_key_hash,
        ) {
            Ok(()) => {}
            Err(ring::error::Unspecified) => {
                log::warn!("{} {} -> [invalid key]", req.method(), req.uri());
                let mut resp = Response::new(body::empty());
                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                return resp;
            }
        }
    }

    let uri = match extract_uri_from_path(req.uri()) {
        None => {
            log::warn!("{} {} -> [missing url]", req.method(), req.uri());
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            return resp;
        }
        Some(Err((e, unparsed))) => {
            log::warn!(
                "{} {} -> [invalid url] {:?} {}",
                req.method(),
                req.uri(),
                unparsed,
                e
            );
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            return resp;
        }
        Some(Ok(u)) => u,
    };

    let orig_method = req.method().clone();
    let orig_uri = mem::replace(req.uri_mut(), uri);
    let mut resp = match state.client.request(req).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("{} {} -> [proxy error] {}", orig_method, orig_uri, e);
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            return resp;
        }
    };

    log::info!("{} {} -> [success]", orig_method, orig_uri);
    resp.headers_mut()
        .append(header::ACCESS_CONTROL_ALLOW_ORIGIN, ANY);
    resp.map(|body| body.boxed())
}
