use hyper::body::{Body, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub type ProxyClient = Client<HttpsConnector<HttpConnector>, Incoming>;

pub fn make_client() -> Result<ProxyClient, io::Error> {
    Ok(Client::builder(TokioExecutor::new()).build(
        HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
    ))
}

pub async fn run_simple_server<S, F, B>(
    addr: SocketAddr,
    state: S,
    handle_req: F,
) -> Result<(), io::Error>
where
    S: Send + Sync + 'static,
    F: for<'s> ServiceFn<'s, Request<Incoming>, S, Response<B>> + Copy + Send + 'static,
    B: Body + Send + 'static,
    <B as Body>::Data: Send,
    <B as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let state = Arc::new(state);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let serve = service_fn(move |req| {
                let state = Arc::clone(&state);
                async move { Ok::<_, Infallible>(handle_req(req, &state).await) }
            });

            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, serve)
                .await
            {
                log::error!("Error serving connection: {}", e);
            }
        });
    }
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
