use std::future::Future;

/// Like futures::future::select_ok, but evaluates each future sequentially, instead of in parallel.
pub async fn first_ok<T, E, F>(iter: impl IntoIterator<Item = F>) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    let mut last_error = None;
    for fut in iter {
        match fut.await {
            Ok(x) => return Ok(x),
            Err(e) => last_error = Some(e),
        }
    }
    match last_error {
        Some(e) => Err(e),
        None => panic!("select_ok: no elements"),
    }
}
