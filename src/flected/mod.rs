use crate::err::Error;
use crate::flected::routes::respond_to_request;
use crate::http;

mod as_ref;
mod body;
mod file;
pub mod opt;
mod routes;

pub async fn main(options: opt::Options) -> Result<(), Error> {
    let opt::Options { listen } = options;

    http::run_simple_server(listen, Default::default(), respond_to_request).await?;

    Ok(())
}
