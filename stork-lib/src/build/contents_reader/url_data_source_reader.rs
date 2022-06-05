use crate::{build::errors::DocumentReadError, config::Filetype};
use mime::Mime;
use std::io::Read;

pub(crate) fn read(url: &str) -> Result<(String, Option<Filetype>), DocumentReadError> {
    if cfg!(not(feature = "web-scraping")) {
        Err(DocumentReadError::WebScrapingNotEnabled)
    } else {
        let mut resp =
            reqwest::blocking::get(url).map_err(|_| DocumentReadError::WebPageNotFetched)?;

        let _status = resp.error_for_status_ref().map_err(|error| {
            match error.status().map(|s| s.as_u16()) {
                Some(status_code) => DocumentReadError::WebPageErrorfulStatusCode(status_code),
                None => DocumentReadError::WebPageNotFetched,
            }
        })?;

        let mime: Mime = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .ok_or(DocumentReadError::UnknownContentType)?
            .to_str()
            .map_err(|_| DocumentReadError::UnknownContentType)?
            .parse()
            .map_err(|_| DocumentReadError::UnknownContentType)?;

        let filetype = match (mime.type_(), mime.subtype()) {
            (mime::TEXT, mime::PLAIN) => Some(Filetype::PlainText),
            (mime::TEXT, mime::HTML) => Some(Filetype::HTML),
            // TODO: More?
            _ => None,
        };

        let mut buffer = String::new();
        let _bytes_read = resp.read_to_string(&mut buffer);

        Ok((buffer, filetype))
    }
}
