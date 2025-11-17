use hyper::Uri;
use hyper::http::uri::InvalidUri;
use std::borrow::Cow;
use std::str::FromStr;

pub fn extract_uri_from_path(uri: &Uri) -> Option<Result<Uri, (InvalidUri, Cow<'_, str>)>> {
    let path = uri.path_and_query()?.as_str().strip_prefix('/')?;

    let unparsed = if path.starts_with("https:") || path.starts_with("http:") {
        Cow::Borrowed(path)
    } else {
        // if no protocol provided, default to https:
        Cow::Owned(format!("https://{}", path))
    };

    Some(match Uri::from_str(&unparsed) {
        Ok(u) => Ok(u),
        Err(e) => Err((e, unparsed)),
    })
}
