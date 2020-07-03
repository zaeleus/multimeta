use reqwest::Url;

pub use self::melon::MelonExtractor;
pub use self::mora::MoraExtractor;
pub use self::up_front_works::UpFrontWorksExtractor;

pub mod melon;
pub mod mora;
pub mod up_front_works;

pub use crate::models::Album;

pub type Result<T> = std::result::Result<T, ExtractionError>;

#[derive(Debug)]
pub enum ExtractionError {
    Factory,
    Fetch(reqwest::Error),
    InvalidUrl(&'static str),
    InvalidDocument,
    MissingField(&'static str),
    InvalidField(&'static str),
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
    use reqwest::Url;

    use super::factory;

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
