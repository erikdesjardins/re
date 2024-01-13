use crate::err::Error;
use http_body_util::BodyExt;
use hyper::body::Body;
use memmap2::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn write_to_mmap<B, E>(
    mut body: impl Body<Data = B, Error = E> + Unpin,
) -> Result<Mmap, Error>
where
    B: AsRef<[u8]>,
    Error: From<E>,
{
    let file = tempfile()?;

    let mut file = File::from_std(file);
    while let Some(frame) = body.frame().await {
        let frame = frame?;
        if let Some(bytes) = frame.data_ref() {
            file.write_all(bytes.as_ref()).await?;
        }
    }
    let file = file.into_std().await;

    // safety: this is an unlinked, exclusive-access temporary file,
    // so it cannot be modified or truncated by anyone else
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(mmap)
}
