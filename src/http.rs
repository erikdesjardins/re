use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

pub fn make_http_client() -> Client<HttpsConnector<HttpConnector>> {
    Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
    )
}

pub async fn run_simple_server<S, F, B, E>(
    addr: SocketAddr,
    state: S,
    handle_req: F,
) -> Result<(), hyper::Error>
where
    S: Send + Sync + 'static,
    F: for<'s> ServiceFn<'s, Request<Body>, S, Result<Response<B>, E>> + Copy + Send + 'static,
    B: HttpBody + Send + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    E: Into<Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
{
    let state = Arc::new(state);
    let make_svc = make_service_fn(move |_| {
        let state = Arc::clone(&state);
        let svc = service_fn(move |req| {
            let state = Arc::clone(&state);
            async move { handle_req(req, &state).await }
        });
        async move { Ok::<_, Infallible>(svc) }
    });

    Server::try_bind(&addr)?.serve(make_svc).await?;

    Ok(())
}

// Work around the lack of HKT bounds.
// Because the future will borrow from the state argument, we need to write bounds like this:
// ```
// where
//     F: for<'s> FnOnce(Request<Body>, &'s S) -> Fut<'s>
//     Fut<'s>: Future<Output = Result<Response<B>, E>> + 's
// ```
// Which can't currently be done. Instead, factor both bounds out to a dedicated trait,
// which is implemented for all matching functions.
pub trait ServiceFn<'s, T, S, R>
where
    Self: FnOnce(T, &'s S) -> Self::Fut,
    Self::Fut: Future<Output = R> + Send + 's,
    S: 's,
{
    type Fut;
}

impl<'s, T, S, R, F, Fut> ServiceFn<'s, T, S, R> for F
where
    F: FnOnce(T, &'s S) -> Fut,
    Fut: Future<Output = R> + Send + 's,
    S: 's,
{
    type Fut = Fut;
}
