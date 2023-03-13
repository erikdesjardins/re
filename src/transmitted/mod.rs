use crate::err::Error;
use crate::http::{make_http_client, run_simple_server};
use crate::transmitted::routes::{respond_to_request, State};

pub mod opt;
mod path;
mod routes;

pub async fn main(options: opt::Options) -> Result<(), Error> {
    let opt::Options { listen, secret_key } = options;

    let state = State {
        client: make_http_client(),
        secret_key,
    };

    run_simple_server(listen, state, respond_to_request).await?;

    Ok(())
}
