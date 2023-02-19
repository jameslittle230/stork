use crate::build_config;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum UrlReadError {
    #[error("Stork wasn't built with web scraping functionality enabled.")]
    FeatureNotEnabled,

    #[error("Error fetching webpage.")]
    WebPageNotFetched,

    #[error("Got status code {0} indicating page couldn't be read")]
    BadStatusCode(u16),

    #[error("Couldn't determine a content-type.")]
    UnknownContentType,
}

#[cfg(not(feature = "build-remote-fetch"))]
pub(crate) fn read(url: &str) -> Result<(String, Option<build_config::Filetype>), UrlReadError> {
    Err(UrlReadError::FeatureNotEnabled)
}

#[cfg(feature = "build-remote-fetch")]
pub(crate) fn read(url: &str) -> Result<(String, Option<build_config::Filetype>), UrlReadError> {
    use mime::Mime;
    use std::io::Read;

    let mut resp = reqwest::blocking::get(url).map_err(|_e| UrlReadError::WebPageNotFetched)?;

    let _status =
        resp.error_for_status_ref()
            .map_err(|error| match error.status().map(|s| s.as_u16()) {
                Some(status_code) => UrlReadError::BadStatusCode(status_code),
                None => UrlReadError::WebPageNotFetched,
            })?;

    let mime: Mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .ok_or(UrlReadError::UnknownContentType)?
        .to_str()
        .map_err(|_| UrlReadError::UnknownContentType)?
        .parse()
        .map_err(|_| UrlReadError::UnknownContentType)?;

    let filetype = match (mime.type_(), mime.subtype()) {
        (mime::TEXT, mime::PLAIN) => Some(build_config::Filetype::PlainText),
        (mime::TEXT, mime::HTML) => Some(build_config::Filetype::HTML),
        // TODO: Add more well-known mime types? Markdown? SRT?
        _ => None,
    };

    let mut buffer = String::new();
    let _bytes_read = resp.read_to_string(&mut buffer);

    Ok((buffer, filetype))
}
