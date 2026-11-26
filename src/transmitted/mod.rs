use crate::err::Error;
use crate::http;
use crate::transmitted::routes::{State, respond_to_request};

pub mod opt;
mod path;
mod routes;
pub mod secret;

pub async fn main(options: opt::Options) -> Result<(), Error> {
    let opt::Options {
        listen,
        key: opt::KeyOptions {
            secret_key,
            no_secret_key,
        },
    } = options;

    let state = State {
        client: http::make_client()?,
        secret_key_hash: match (secret_key, no_secret_key) {
            (Some(secret_key), _) => Some(secret::Key::new(secret_key.as_bytes())?),
            (None, true) => None,
            (None, false) => unreachable!("no secret key but no no_secret_key"),
        },
    };

    http::run_simple_server(listen, state, respond_to_request).await?;

    Ok(())
}
