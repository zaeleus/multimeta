use url::Url;

pub use self::melon::MelonExtractor;
pub use self::mora::MoraExtractor;

pub mod melon;
pub mod mora;

pub use models::Album;

#[derive(Debug, Eq, PartialEq)]
pub enum ExtractionError {
    Factory(&'static str),
    Fetch,
    Parse(&'static str),
    Url(&'static str),
}

pub trait Extractor {
    fn extract(&self) -> Result<Album, ExtractionError>;
}

pub fn factory(url: &Url) -> Result<Box<Extractor>, ExtractionError> {
    if MelonExtractor::matches(&url) {
        Ok(Box::new(MelonExtractor::new(&url)?))
    } else if MoraExtractor::matches(&url) {
        Ok(Box::new(MoraExtractor::new(&url)?))
    } else {
        Err(ExtractionError::Factory("failed to match url to a suitable extractor"))
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::factory;

    #[test]
    fn test_factory() {
        let url = Url::parse("http://www.melon.com/album/detail.htm?albumId=10141232").unwrap();
        assert!(factory(&url).is_ok());

        let url = Url::parse("http://mora.jp/package/43000001/4547366347050/").unwrap();
        assert!(factory(&url).is_ok());

        let url = Url::parse("http://www.google.com/").unwrap();
        assert!(factory(&url).is_err());
    }
}
