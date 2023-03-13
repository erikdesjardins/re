use crate::directed::redir::Rules;
use crate::directed::routes::{respond_to_request, State};
use crate::err::Error;
use crate::http::{make_http_client, run_simple_server};

mod file;
pub mod opt;
mod redir;
mod routes;

pub async fn main(options: opt::Options) -> Result<(), Error> {
    let opt::Options { listen, from, to } = options;

    let state = State {
        client: make_http_client(),
        rules: Rules::zip(from, to)?,
    };

    run_simple_server(listen, state, respond_to_request).await?;

    Ok(())
}
