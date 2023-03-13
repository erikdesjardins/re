mod as_ref;
mod body;
mod file;
pub mod opt;
mod routes;
mod server;

pub async fn main(options: opt::Options) -> Result<(), crate::err::Error> {
    let opt::Options { listen } = options;

    server::run(&listen).await?;

    Ok(())
}
