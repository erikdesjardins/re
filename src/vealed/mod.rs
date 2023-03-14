mod forwarder;
pub mod opt;

pub async fn main(options: opt::Options) -> Result<(), std::io::Error> {
    let opt::Options { listen, to } = options;

    forwarder::run(listen, &to).await?;

    Ok(())
}
