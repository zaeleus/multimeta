pub mod melon;
pub mod mora;
pub mod up_front_works;

pub use self::{melon::MelonExtractor, mora::MoraExtractor, up_front_works::UpFrontWorksExtractor};

use std::{error, fmt, io};

use url::Url;

use crate::models::Album;

pub type Result<T> = std::result::Result<T, ExtractionError>;

#[derive(Debug)]
pub enum ExtractionError {
    Factory,
    FetchRequest(Box<ureq::Error>),
    FetchBody(io::Error),
    InvalidUrl(&'static str),
    InvalidDocument,
    MissingField(&'static str),
    InvalidField(&'static str),
}

impl error::Error for ExtractionError {}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factory => f.write_str("could not construct an extractor from the url"),
            Self::FetchRequest(e) => write!(f, "{}", e),
            Self::FetchBody(e) => write!(f, "{}", e),
            Self::InvalidUrl(key) => write!(f, "invalid url: missing {}", key),
            Self::InvalidDocument => f.write_str("could not parse document"),
            Self::MissingField(field) => write!(f, "missing field: {}", field),
            Self::InvalidField(field) => write!(f, "invalid field: {}", field),
        }
    }
}

pub trait Extractor {
    fn extract(&self) -> self::Result<Album>;
}

pub fn factory(url: &Url) -> self::Result<Box<dyn Extractor>> {
    if MelonExtractor::matches(&url) {
        Ok(Box::new(MelonExtractor::from_url(&url)?))
    } else if MoraExtractor::matches(&url) {
        Ok(Box::new(MoraExtractor::from_url(&url)?))
    } else if UpFrontWorksExtractor::matches(&url) {
        Ok(Box::new(UpFrontWorksExtractor::from_url(&url)?))
    } else {
        Err(ExtractionError::Factory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory() {
        let url = Url::parse("http://www.melon.com/album/detail.htm?albumId=10141232").unwrap();
        assert!(factory(&url).is_ok());

        let url = Url::parse("http://mora.jp/package/43000001/4547366347050/").unwrap();
        assert!(factory(&url).is_ok());

        let url = Url::parse("http://www.up-front-works.jp/release/detail/EPCE-7387/").unwrap();
        assert!(factory(&url).is_ok());

        let url = Url::parse("http://www.google.com/").unwrap();
        assert!(factory(&url).is_err());
    }
}
