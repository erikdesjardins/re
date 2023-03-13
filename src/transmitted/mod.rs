pub mod opt;
mod path;
mod routes;
mod server;

pub async fn main(options: opt::Options) -> Result<(), crate::err::Error> {
    let opt::Options { listen, secret_key } = options;

    server::run(listen, secret_key).await?;

    Ok(())
}
