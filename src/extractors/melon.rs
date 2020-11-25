use chrono::NaiveDate;
use log::warn;
use select::{document::Document, predicate::Class};
use serde::Deserialize;
use url::Url;

use crate::{
    extractors::{self, ExtractionError, Extractor},
    models::{album, song, Album, Name},
};

static HOST: &str = "www.melon.com";
static HTML_ENDPOINT: &str = "https://www.melon.com/album/detail.htm";
static JSON_ENDPOINT: &str = "https://www.melon.com/webplayer/getContsInfo.json";

static COUNTRY: &str = "KR";
static LOCALE: &str = "ko";

pub struct MelonExtractor {
    album_id: String,
}

impl MelonExtractor {
    pub fn matches(url: &Url) -> bool {
        url.host_str().map(|h| h == HOST).unwrap_or(false)
    }

    pub fn from_url(url: &Url) -> extractors::Result<MelonExtractor> {
        parse_album_id(url).map(MelonExtractor::new)
    }

    pub fn new<I>(album_id: I) -> MelonExtractor
    where
        I: Into<String>,
    {
        MelonExtractor {
            album_id: album_id.into(),
        }
    }

    fn fetch_html(&self) -> extractors::Result<String> {
        let params = [("albumId", &self.album_id)];
        let url = Url::parse_with_params(HTML_ENDPOINT, &params).unwrap();
        fetch(&url.into_string())
    }

    fn fetch_json(&self) -> extractors::Result<String> {
        let params = [("contsType", "A"), ("contsIds", &self.album_id)];
        let url = Url::parse_with_params(JSON_ENDPOINT, &params).unwrap();
        fetch(&url.into_string())
    }
}

impl Extractor for MelonExtractor {
    fn extract(&self) -> extractors::Result<Album> {
        let html = self.fetch_html()?;
        let json = self.fetch_json()?;
        parse(&self.album_id, &html, &json)
    }
}

fn fetch(url: &str) -> extractors::Result<String> {
    ureq::get(url)
        .call()
        .map_err(ExtractionError::FetchRequest)
        .and_then(|r| r.into_string().map_err(ExtractionError::FetchBody))
}

fn parse(album_id: &str, html: &str, json: &str) -> extractors::Result<Album> {
    let builder = album::Builder::new();

    let builder = builder
        .set_country(COUNTRY)
        .set_url(&format!("{}?albumId={}", HTML_ENDPOINT, album_id));

    let builder = parse_html(html, builder)?;
    let builder = parse_json(json, builder)?;

    Ok(builder.build())
}

fn parse_html(html: &str, builder: album::Builder) -> extractors::Result<album::Builder> {
    let document = Document::from(html);

    let mut node = document.find(Class("gubun"));

    let raw_kind = node
        .next()
        .ok_or(ExtractionError::MissingField("album kind"))
        .map(|n| n.text())?;
    let raw_kind = raw_kind.trim();
    // Remove surrounding brackets from text.
    let raw_kind = &raw_kind[1..raw_kind.len() - 1];
    let kind = parse_album_kind(raw_kind)?;

    let builder = builder.set_kind(kind);

    Ok(builder)
}

fn parse_json(json: &str, builder: album::Builder) -> extractors::Result<album::Builder> {
    let root: Root = serde_json::from_str(json).map_err(|_| ExtractionError::InvalidDocument)?;

    let songs = root.conts_list;

    let builder = if let Some(song) = songs.first() {
        let raw_name = normalize_name(&song.album_name_web_list);
        let name = Name::new(raw_name, LOCALE, true, true);

        builder
            .set_released_on(&parse_release_date(&song.issue_date)?)
            .set_artwork_url(&parse_artwork_url(&song.album_img_path))
            .add_name(name)
    } else {
        return Err(ExtractionError::MissingField("songs"));
    };

    let builder = parse_songs(&songs, builder)?;

    Ok(builder)
}

fn parse_songs(
    songs: &[RawSong],
    mut builder: album::Builder,
) -> extractors::Result<album::Builder> {
    for song in songs {
        let raw_name = normalize_name(&song.song_name);
        let name = Name::new(raw_name, LOCALE, true, true);

        let position = parse_position(&song.track_no)?;
        let duration = song.play_time;

        let song = song::Builder::new()
            .set_position(position)
            .set_duration(duration)
            .add_name(name)
            .build();

        builder = builder.add_song(song);
    }

    Ok(builder)
}

fn parse_album_id(url: &Url) -> extractors::Result<String> {
    url.query_pairs()
        .find(|&(ref k, _)| k == "albumId")
        .map(|(_, v)| v.into_owned())
        .ok_or(ExtractionError::InvalidUrl("albumId"))
}

fn parse_album_kind(s: &str) -> extractors::Result<album::Kind> {
    match s {
        "싱글" => Ok(album::Kind::Single),
        "OST" => {
            // "OST" is not guaranteed, but is very likely, to be a single.
            warn!("assuming album kind 'OST' as 'single'");
            Ok(album::Kind::Single)
        }
        "리믹스" => {
            warn!("assuming album kind '리믹스' as 'single'");
            Ok(album::Kind::Single)
        }
        "EP" => Ok(album::Kind::Ep),
        "정규" => Ok(album::Kind::Lp),
        "옴니버스" => {
            // "Omnibus" is probably either an EP or LP, but since it's
            // typically a collection, assume it's an album.
            warn!("assuming album kind '옴니버스' as 'LP'");
            Ok(album::Kind::Lp)
        }
        "베스트" => {
            // "Best" is probably a compilation album.
            warn!("assuming album kind '베스트' as 'LP'");
            Ok(album::Kind::Lp)
        }
        _ => Err(ExtractionError::InvalidField("album kind")),
    }
}

fn parse_artwork_url(s: &str) -> String {
    let end = s.len() - 4;
    let segment = &s[..end];
    format!("https://static.melon.co.kr{}_org.jpg", segment)
}

fn parse_position(s: &str) -> extractors::Result<i32> {
    s.parse()
        .map_err(|_| ExtractionError::InvalidField("position"))
}

fn parse_release_date(s: &str) -> extractors::Result<String> {
    NaiveDate::parse_from_str(s, "%Y%m%d")
        .map(|d| d.format("%F").to_string())
        .map_err(|_| ExtractionError::InvalidField("release date"))
}

fn normalize_name(name: &str) -> String {
    name.replace("`", "'")
        .replace("‘", "'")
        .replace("’", "'")
        .replace("′", "'")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
        .replace(" Of ", " of ")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    conts_list: Vec<RawSong>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSong {
    album_img_path: String,
    album_name_web_list: String,
    issue_date: String,
    play_time: i32,
    song_name: String,
    track_no: String,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_matches() {
        let url = Url::parse("http://www.melon.com/album/detail.htm?albumId=10141232").unwrap();
        assert!(MelonExtractor::matches(&url));

        let url = Url::parse("http://www.melon.com/chart/index.htm").unwrap();
        assert!(MelonExtractor::matches(&url));

        let url = Url::parse("http://www.melon.com/").unwrap();
        assert!(MelonExtractor::matches(&url));

        let url = Url::parse("https://www.google.com/").unwrap();
        assert!(!MelonExtractor::matches(&url))
    }

    #[test]
    fn test_parse_html() {
        let html = fs::read_to_string("tests/fixtures/melon-10123637.html").unwrap();
        let json = fs::read_to_string("tests/fixtures/melon-10123637.json").unwrap();

        let album = parse("10123637", &html, &json).unwrap();

        assert_eq!(album.kind, album::Kind::Single);
        assert_eq!(album.country, "KR");
        assert_eq!(album.released_on, "2017-12-28");
        assert_eq!(
            album.artwork_url,
            Some(String::from(
                "https://static.melon.co.kr/cm/album/images/101/23/637/10123637_org.jpg"
            )),
        );
        assert_eq!(
            album.url,
            "https://www.melon.com/album/detail.htm?albumId=10123637"
        );

        assert_eq!(album.names.len(), 1);
        assert_eq!(&album.names[0], &Name::new("Chuu", "ko", true, true));

        assert_eq!(album.songs.len(), 2);

        let song = &album.songs[0];
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 195);
        assert_eq!(song.names.len(), 1);
        assert_eq!(
            &song.names[0],
            &Name::new("Heart Attack (츄)", "ko", true, true)
        );

        let song = &album.songs[1];
        assert_eq!(song.position, 2);
        assert_eq!(song.duration, 197);
        assert_eq!(song.names.len(), 1);
        assert_eq!(
            &song.names[0],
            &Name::new("Girl's Talk (이브, 츄)", "ko", true, true)
        );
    }

    #[test]
    fn test_parse_html_with_empty_document() {
        let builder = album::Builder::new();
        assert!(parse_json("<html />", builder).is_err());
    }

    #[test]
    fn test_parse_json_with_empty_root() {
        let builder = album::Builder::new();
        assert!(parse_json("{}", builder).is_err());
    }

    #[test]
    fn test_parse_album_id() {
        let url = Url::parse("https://www.melon.com/album/detail.htm?albumId=10141232").unwrap();
        assert_eq!(parse_album_id(&url).unwrap(), "10141232");

        let url = Url::parse("https://www.melon.com/album/detail.html").unwrap();
        assert!(parse_album_id(&url).is_err());
    }

    #[test]
    fn test_parse_album_kind() {
        assert_eq!(parse_album_kind("싱글").unwrap(), album::Kind::Single);
        assert_eq!(parse_album_kind("EP").unwrap(), album::Kind::Ep);
        assert_eq!(parse_album_kind("정규").unwrap(), album::Kind::Lp);
        assert_eq!(parse_album_kind("OST").unwrap(), album::Kind::Single);
        assert_eq!(parse_album_kind("리믹스").unwrap(), album::Kind::Single);
        assert_eq!(parse_album_kind("옴니버스").unwrap(), album::Kind::Lp);

        // https://www.melon.com/album/detail.htm?albumId=10404130
        assert_eq!(parse_album_kind("베스트").unwrap(), album::Kind::Lp);

        assert!(parse_album_kind("foo").is_err());
    }

    #[test]
    fn test_parse_artwork_url() {
        let actual = parse_artwork_url("/cm/album/images/101/23/637/10123637.jpg");
        let expected = "https://static.melon.co.kr/cm/album/images/101/23/637/10123637_org.jpg";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_release_date() {
        assert_eq!(parse_release_date("20171228").unwrap(), "2017-12-28");
        assert!(parse_release_date("2017").is_err());
    }

    #[test]
    fn test_normalize_name() {
        // https://www.melon.com/album/detail.htm?albumId=10123637
        assert_eq!(
            normalize_name("Girl`s Talk (이브, 츄)"),
            "Girl's Talk (이브, 츄)"
        );

        // https://www.melon.com/album/detail.htm?albumId=598055
        assert_eq!(normalize_name("I Don’t Care"), "I Don't Care");

        // https://www.melon.com/album/detail.htm?albumId=10288337
        assert_eq!(
            normalize_name("3YE 1st Digital Single ‘DMT`"),
            "3YE 1st Digital Single 'DMT'"
        );

        // https://www.melon.com/album/detail.htm?albumId=10353029
        assert_eq!(normalize_name("Let′s Go Everywhere"), "Let's Go Everywhere");

        // https://www.melon.com/album/detail.htm?albumId=10310489
        assert_eq!(
            normalize_name("&#34;개 같은 하루 (with TTG)&#34; OST"),
            r#""개 같은 하루 (with TTG)" OST"#,
        );

        // https://www.melon.com/album/detail.htm?albumId=10309095
        assert_eq!(normalize_name("서핑해 (Surfin&#39;)"), "서핑해 (Surfin')");

        // https://www.melon.com/album/detail.htm?albumId=10074419
        let actual = normalize_name("Love Don`t Hurt (Feat. Amber Of f(x))");
        let expected = "Love Don't Hurt (Feat. Amber of f(x))";
        assert_eq!(actual, expected);
    }
}
