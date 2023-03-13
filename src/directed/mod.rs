#[macro_use]
mod macros;

mod file;
pub mod opt;
mod redir;
mod routes;
mod server;

use redir::Rules;

pub async fn main(options: opt::Options) -> Result<(), crate::err::Error> {
    let opt::Options { listen, from, to } = options;

    server::run(&listen, Rules::zip(from, to)?).await?;

    Ok(())
}
