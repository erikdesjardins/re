use crate::flected::body::ArcBody;
use crate::flected::file::write_to_mmap;
use crate::flected::routes::State;
use headers::{AcceptRanges, ContentRange, HeaderMapExt, Range};
use hyper::body::Incoming;
use hyper::header::HOST;
use hyper::{Request, Response, StatusCode};
use std::collections::Bound;
use std::sync::Arc;

pub async fn get(req: Request<Incoming>, state: &State) -> Response<ArcBody> {
    let file = state.files.read().await.get(req.uri().path()).cloned();
    match file {
        Some(file) => match req
            .headers()
            .typed_get::<Range>()
            .and_then(|r| r.satisfiable_ranges(file.len() as u64).next())
        {
            Some((start, end)) => {
                let file_len = file.len();
                let start_inclusive = match start {
                    Bound::Included(start) => start as usize,
                    Bound::Excluded(start) => start as usize + 1,
                    Bound::Unbounded => 0,
                };
                let end_exclusive = match end {
                    Bound::Included(end) => end as usize + 1,
                    Bound::Excluded(end) => end as usize,
                    Bound::Unbounded => file_len,
                };
                match ArcBody::from_arc_with_range(file, start_inclusive..end_exclusive) {
                    Ok(body) => {
                        log::info!(
                            "GET {} -> [found range {}..{} bytes of {}]",
                            req.uri(),
                            start_inclusive,
                            end_exclusive,
                            file_len
                        );
                        let mut resp = Response::new(body);
                        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                        resp.headers_mut().typed_insert(
                            ContentRange::bytes(
                                (start_inclusive as u64)..(end_exclusive as u64),
                                file_len as u64,
                            )
                            .unwrap(),
                        );
                        resp
                    }
                    Err(_) => {
                        log::info!("GET {} -> [bad range]", req.uri());
                        let mut resp = Response::new(ArcBody::empty());
                        *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                        resp.headers_mut()
                            .typed_insert(ContentRange::unsatisfied_bytes(file_len as u64));
                        resp
                    }
                }
            }
            None => {
                log::info!("GET {} -> [found {} bytes]", req.uri(), file.len());
                let mut resp = Response::new(ArcBody::from_arc(file));
                resp.headers_mut().typed_insert(AcceptRanges::bytes());
                resp
            }
        },
        None => {
            log::info!("GET {} -> [not found]", req.uri());
            let path = req.uri().path().trim_start_matches('/');
            let host = req
                .headers()
                .get(HOST)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("example.com");
            let mut resp = Response::new(ArcBody::new(
                format!(concat!(
                    "<!DOCTYPE html>",
                    "<html>",
                    "<head></head>",
                    "<body>",
                    "<code>curl -o /dev/null -X POST {host}/{path} --data-binary @- < {path}</code>",
                    "<p/>",
                    "<span id='info'>or </span>",
                    "<input",
                    " type='file'",
                    " onchange='disabled = true, info.replaceWith(`uploading...`), fetch(``, {{ method: `POST`, body: files[0] }}).then(() => this.replaceWith(`done`))'",
                    "/>",
                    "</body>",
                    "</html>",
                ), path = path, host = host)
            ));
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
    }
}

pub async fn post(req: Request<Incoming>, state: &State) -> Response<ArcBody> {
    log::info!("POST {} -> [start upload]", req.uri());
    let (parts, body) = req.into_parts();
    let file = match write_to_mmap(body).await {
        Ok(f) => f,
        Err(e) => {
            log::warn!("POST {} -> [upload error] {}", parts.uri, e);
            let mut resp = Response::new(ArcBody::empty());
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp;
        }
    };
    log::info!("POST {} -> [uploaded {} bytes]", parts.uri, file.len());
    state
        .files
        .write()
        .await
        .insert(parts.uri.path().to_string(), Arc::new(file));
    Response::new(ArcBody::empty())
}

pub async fn delete(req: Request<Incoming>, state: &State) -> Response<ArcBody> {
    let file = state.files.write().await.remove(req.uri().path());
    match file {
        Some(file) => {
            log::info!("DELETE {} -> [deleted {} bytes]", req.uri(), file.len());
            Response::new(ArcBody::empty())
        }
        None => {
            log::info!("DELETE {} -> [not found]", req.uri());
            let mut resp = Response::new(ArcBody::empty());
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
    }
}
