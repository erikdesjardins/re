use crate::directed::redir::Rules;
use crate::directed::routes::{State, respond_to_request};
use crate::err::Error;
use crate::http;

pub mod opt;
mod redir;
mod routes;

pub async fn main(options: opt::Options) -> Result<(), Error> {
    let opt::Options { listen, from, to } = options;

    let state = State {
        client: http::make_client()?,
        rules: Rules::zip(from, to)?,
    };

    http::run_simple_server(listen, state, respond_to_request).await?;

    Ok(())
}
