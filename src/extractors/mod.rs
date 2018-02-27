use url::Url;

pub use self::melon::MelonExtractor;

pub mod melon;

pub use models::Album;

#[derive(Debug, Eq, PartialEq)]
pub enum ExtractionError {
    Fetch,
    Parse(&'static str),
    Url(&'static str),
}

pub trait Extractor {
    fn extract(&self) -> Result<Album, ExtractionError>;
    fn matches(url: &Url) -> bool;
}
