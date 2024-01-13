use crate::body;
use crate::directed::redir::{Action, Rules};
use crate::err::Error;
use crate::http::ProxyClient;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use tokio::fs::File;

pub struct State {
    pub client: ProxyClient,
    pub rules: Rules,
}

pub async fn respond_to_request(
    mut req: Request<Incoming>,
    state: &State,
) -> Response<BoxBody<Bytes, Error>> {
    match state.rules.try_match(req.uri()) {
        Some(Ok(Action::Http(uri))) => {
            // Proxy this request to the new URL
            let req_uri = req.uri().clone();
            req.uri_mut().clone_from(&uri);
            match state.client.request(req).await {
                Ok(resp) => {
                    log::info!("{} -> {}", req_uri, uri);
                    resp.map(|body| body.map_err(Error::from).boxed())
                }
                Err(e) => {
                    log::warn!("{} -> [proxy error] {} : {}", req_uri, uri, e);
                    let mut resp = Response::new(body::empty());
                    *resp.status_mut() = StatusCode::BAD_GATEWAY;
                    resp
                }
            }
        }
        Some(Ok(Action::File { path, fallback })) => {
            // Proxy this request to the filesystem
            let found_file = match File::open(&path).await {
                Ok(file) => Ok((path, file)),
                Err(e) => match fallback {
                    Some(fallback) => match File::open(&fallback).await {
                        Ok(file) => Ok((fallback, file)),
                        Err(_) => Err((path, e)),
                    },
                    None => Err((path, e)),
                },
            };
            match found_file {
                Ok((found_path, file)) => {
                    log::info!("{} -> {}", req.uri(), found_path.display());
                    Response::new(body::from_file(file).map_err(Error::from).boxed())
                }
                Err((path, e)) => {
                    log::warn!("{} -> [file error] {} : {}", req.uri(), path.display(), e);
                    let mut resp = Response::new(body::empty());
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    resp
                }
            }
        }
        Some(Ok(Action::Status(status))) => {
            log::info!("{} -> {}", req.uri(), status);
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = status;
            resp
        }
        Some(Err(e)) => {
            log::warn!("{} -> [internal error] : {}", req.uri(), e);
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            resp
        }
        None => {
            log::warn!("{} -> [no match]", req.uri());
            let mut resp = Response::new(body::empty());
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            resp
        }
    }
}
