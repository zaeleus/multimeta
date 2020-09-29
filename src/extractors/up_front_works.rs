use chrono::NaiveDate;
use reqwest::Url;
use select::{
    document::Document,
    predicate::{self, Class, Descendant},
};

use crate::{
    extractors::{self, ExtractionError, Extractor},
    models::{album, Album, Name, SongBuilder},
};

static HOST: &str = "www.up-front-works.jp";

static BASE_URL: &str = "http://www.up-front-works.jp/release/detail";

static COUNTRY: &str = "JP";
static LOCALE: &str = "ja";

pub struct UpFrontWorksExtractor {
    album_id: String,
}

impl UpFrontWorksExtractor {
    pub fn matches(url: &Url) -> bool {
        url.host_str().map(|h| h == HOST).unwrap_or(false)
    }

    pub fn from_url(url: &Url) -> extractors::Result<UpFrontWorksExtractor> {
        parse_album_id(url).map(UpFrontWorksExtractor::new)
    }

    pub fn new<I>(album_id: I) -> UpFrontWorksExtractor
    where
        I: Into<String>,
    {
        UpFrontWorksExtractor {
            album_id: album_id.into(),
        }
    }

    fn fetch(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/", BASE_URL, self.album_id);
        reqwest::blocking::get(&url).and_then(|r| r.text())
    }
}

impl Extractor for UpFrontWorksExtractor {
    fn extract(&self) -> extractors::Result<Album> {
        let html = self.fetch().map_err(ExtractionError::Fetch)?;
        parse(&self.album_id, &html)
    }
}

fn parse(album_id: &str, html: &str) -> extractors::Result<Album> {
    let url = format!("{}/{}/", BASE_URL, album_id);

    let builder = album::Builder::new().set_country(COUNTRY).set_url(&url);

    let builder = parse_html(html, builder)?;

    Ok(builder.build())
}

fn parse_html(html: &str, builder: album::Builder) -> extractors::Result<album::Builder> {
    let document = Document::from(html);

    let name = document
        .find(Class("product_title"))
        .next()
        .ok_or_else(|| ExtractionError::MissingField("name"))
        .map(|n| n.text())
        .map(|n| Name::new(n, LOCALE, true, true))?;

    let mut meta_node = document.find(Descendant(Class("data1"), Class("columnB")));

    let kind = meta_node
        .next()
        .ok_or_else(|| ExtractionError::MissingField("kind"))
        .map(|n| n.text())
        .and_then(|kind| parse_kind(&kind))?;

    let released_on = meta_node
        .next()
        .ok_or_else(|| ExtractionError::MissingField("release date"))
        .map(|n| n.text())
        .and_then(|date| parse_release_date(&date))?;

    let builder = builder
        .set_kind(kind)
        .set_released_on(&released_on)
        .add_name(name);

    let builder = parse_songs(&document, builder)?;

    Ok(builder)
}

fn parse_songs(
    document: &Document,
    mut builder: album::Builder,
) -> extractors::Result<album::Builder> {
    let table = document
        .find(Class("data2"))
        .next()
        .ok_or_else(|| ExtractionError::MissingField("songs"))?;

    let rows = table
        .find(predicate::Name("tr"))
        // skip header
        .skip(1);

    for (i, row) in rows.enumerate() {
        // skip odd rows with track artist
        if i % 2 != 0 {
            continue;
        }

        let mut cells = row.find(predicate::Name("td"));

        let position = cells
            .next()
            .ok_or_else(|| ExtractionError::MissingField("songs[_].track_number"))
            .map(|n| n.text())
            .and_then(|s| parse_position(&s))?;

        let name = cells
            .next()
            .ok_or_else(|| ExtractionError::MissingField("songs[_].name"))
            .map(|n| n.text())
            .map(|n| Name::new(n, LOCALE, true, true))?;

        let duration = cells
            .next()
            .ok_or_else(|| ExtractionError::MissingField("songs[_].duration"))
            .map(|n| n.text())
            .and_then(|s| parse_duration(&s))?;

        let song = SongBuilder::new()
            .set_position(position)
            .set_duration(duration)
            .add_name(name)
            .build();

        builder = builder.add_song(song);
    }

    Ok(builder)
}

fn parse_position(s: &str) -> extractors::Result<i32> {
    s.parse()
        .map_err(|_| ExtractionError::InvalidField("position"))
}

fn parse_duration(s: &str) -> extractors::Result<i32> {
    let mut pieces = s.splitn(2, ':');

    let minutes: i32 = pieces
        .next()
        .ok_or_else(|| ExtractionError::MissingField("duration.minutes"))
        .and_then(|s| {
            s.parse()
                .map_err(|_| ExtractionError::InvalidField("duration.minutes"))
        })?;

    let seconds: i32 = pieces
        .next()
        .ok_or_else(|| ExtractionError::MissingField("duration.seconds"))
        .and_then(|s| {
            s.parse()
                .map_err(|_| ExtractionError::InvalidField("duration.seconds"))
        })?;

    Ok(minutes * 60 + seconds)
}

fn parse_album_id(url: &Url) -> extractors::Result<String> {
    url.path()
        .split('/')
        .filter(|p| !p.is_empty())
        .last()
        .and_then(|id| {
            if id.contains('-') {
                Some(id.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| ExtractionError::InvalidUrl("album ID"))
}

fn parse_kind(s: &str) -> extractors::Result<album::Kind> {
    match s {
        "CDシングル" => Ok(album::Kind::Single),
        "CDミニアルバム" => Ok(album::Kind::Ep),
        "CDアルバム" => Ok(album::Kind::Lp),
        _ => Err(ExtractionError::InvalidField("kind")),
    }
}

fn parse_release_date(s: &str) -> extractors::Result<String> {
    NaiveDate::parse_from_str(s, "%Y/%m/%d")
        .map(|d| d.format("%F").to_string())
        .map_err(|_| ExtractionError::InvalidField("release date"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use reqwest::Url;

    use super::*;

    #[test]
    fn test_matches() {
        let url = Url::parse("http://www.up-front-works.jp/release/detail/EPCE-7387/").unwrap();
        assert!(UpFrontWorksExtractor::matches(&url));

        let url = Url::parse("http://www.up-front-works.jp/artist/").unwrap();
        assert!(UpFrontWorksExtractor::matches(&url));

        let url = Url::parse("http://www.up-front-works.jp/").unwrap();
        assert!(UpFrontWorksExtractor::matches(&url));

        let url = Url::parse("https://www.google.com/").unwrap();
        assert!(!UpFrontWorksExtractor::matches(&url));
    }

    #[test]
    fn test_parse() {
        let html = fs::read_to_string("test/fixtures/up-front-works-epce-7387.html").unwrap();
        let album = parse("EPCE-7387", &html).unwrap();

        assert_eq!(album.kind, album::Kind::Ep);
        assert_eq!(album.country, "JP");
        assert_eq!(album.released_on, "2018-02-07");
        assert!(album.artwork_url.is_none());
        assert_eq!(
            album.url,
            "http://www.up-front-works.jp/release/detail/EPCE-7387/"
        );

        assert_eq!(album.names.len(), 1);
        assert_eq!(
            &album.names[0],
            &Name::new("二十歳のモーニング娘。", "ja", true, true)
        );

        assert_eq!(album.songs.len(), 8);

        let song = &album.songs[0];
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 250);
        assert_eq!(song.names.len(), 1);
        assert_eq!(
            &song.names[0],
            &Name::new(
                "モーニングコーヒー(20th Anniversary Ver.)",
                "ja",
                true,
                true
            )
        );

        let song = &album.songs[7];
        assert_eq!(song.position, 8);
        assert_eq!(song.duration, 250);
        assert_eq!(song.names.len(), 1);
        assert_eq!(
            &song.names[0],
            &Name::new("愛の種(20th Anniversary Ver.)", "ja", true, true)
        );
    }

    #[test]
    fn test_parse_position() {
        assert_eq!(parse_position("1").unwrap(), 1);
        assert_eq!(parse_position("4").unwrap(), 4);
        assert_eq!(parse_position("11").unwrap(), 11);

        assert!(parse_position("").is_err());
        assert!(parse_position("abc").is_err());
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("00:21").unwrap(), 21);
        assert_eq!(parse_duration("01:11").unwrap(), 71);
        assert_eq!(parse_duration("04:10").unwrap(), 250);
        assert_eq!(parse_duration("05:47").unwrap(), 347);

        assert!(parse_duration("").is_err());
        assert!(parse_duration("144").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("aa:bb").is_err());
    }

    #[test]
    fn test_parse_album_id() {
        let url = Url::parse("http://www.up-front-works.jp/release/detail/EPCE-7387/").unwrap();
        assert_eq!(parse_album_id(&url).unwrap(), "EPCE-7387");

        let url = Url::parse("http://www.up-front-works.jp/artist/").unwrap();
        assert!(parse_album_id(&url).is_err());
    }

    #[test]
    fn test_parse_kind() {
        assert_eq!(parse_kind("CDシングル").unwrap(), album::Kind::Single);
        assert_eq!(parse_kind("CDアルバム").unwrap(), album::Kind::Lp);

        assert!(parse_kind("").is_err());
        assert!(parse_kind("album").is_err());
    }

    #[test]
    fn test_parse_release_date() {
        assert_eq!(parse_release_date("2018/02/07").unwrap(), "2018-02-07");

        assert!(parse_release_date("").is_err());
        assert!(parse_release_date("2018").is_err());
    }
}
