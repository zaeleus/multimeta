use chrono::NaiveDate;
use reqwest;
use select::document::Document;
use select::predicate::Class;
use serde_json;
use url::Url;

use extractors::{ExtractionError, Extractor};
use models::{Album, AlbumBuilder, AlbumKind, Name, Song};

static HOST: &'static str = "www.melon.com";
static HTML_ENDPOINT: &'static str = "http://www.melon.com/album/detail.htm";
static JSON_ENDPOINT: &'static str = "http://www.melon.com/webplayer/getContsInfo.json";

static COUNTRY: &'static str = "KR";
static LOCALE: &'static str = "ko";

pub struct MelonExtractor {
    album_id: String,
}

impl MelonExtractor {
    pub fn matches(url: &Url) -> bool {
        url.host_str().map(|h| h == HOST).unwrap_or(false)
    }

    pub fn new(url: &Url) -> Result<MelonExtractor, ExtractionError> {
        Ok(MelonExtractor {
            album_id: parse_album_id(url)?,
        })
    }

    fn fetch_html(&self) -> Result<String, reqwest::Error> {
        let mut url = Url::parse(HTML_ENDPOINT).unwrap();
        url.query_pairs_mut().append_pair("albumId", &self.album_id);
        reqwest::get(&url.into_string()).and_then(|mut r| r.text())
    }

    fn fetch_json(&self) -> Result<String, reqwest::Error> {
        let mut url = Url::parse(JSON_ENDPOINT).unwrap();
        url.query_pairs_mut().append_pair("contsType", "A");
        url.query_pairs_mut().append_pair("contsIds", &self.album_id);
        reqwest::get(&url.into_string()).and_then(|mut r| r.text())
    }
}

impl Extractor for MelonExtractor {
    fn extract(&self) -> Result<Album, ExtractionError> {
        let html = self.fetch_html().map_err(|_| ExtractionError::Fetch)?;
        let json = self.fetch_json().map_err(|_| ExtractionError::Fetch)?;
        parse(&self.album_id, &html, &json)
    }
}

fn parse(album_id: &str, html: &str, json: &str) -> Result<Album, ExtractionError> {
    let builder = AlbumBuilder::new();

    let builder = builder
        .set_country(COUNTRY)
        .set_url(&format!("{}?albumId={}", HTML_ENDPOINT, album_id));

    let builder = parse_html(html, builder)?;
    let builder = parse_json(json, builder)?;

    Ok(builder.build())
}

fn parse_html(html: &str, builder: AlbumBuilder) -> Result<AlbumBuilder, ExtractionError> {
    let document = Document::from(html.as_ref());

    let mut node = document.find(Class("gubun"));

    let raw_kind = node.next()
        .map(|n| n.text())
        .ok_or(ExtractionError::Parse("album kind"))?;
    let raw_kind = raw_kind.trim();
    // Remove surrounding brackets from text.
    let raw_kind = &raw_kind[1..raw_kind.len() - 1];
    let kind = parse_album_kind(raw_kind)?;

    let builder = builder.set_kind(kind);

    Ok(builder)
}

fn parse_json(json: &str, builder: AlbumBuilder) -> Result<AlbumBuilder, ExtractionError> {
    let root: Root = serde_json::from_str(json).map_err(|_| {
        ExtractionError::Parse("malformed JSON")
    })?;

    let songs = root.conts_list;

    let builder = if let Some(song) = songs.first() {
        let raw_name = normalize_name(&song.album_name_web_list);
        let name = Name::new(&raw_name, LOCALE, true, true);

        builder
            .set_released_on(&parse_release_date(&song.issue_date)?)
            .set_artwork_url(&parse_artwork_url(&song.album_img_path))
            .add_name(name)
    } else {
        return Err(ExtractionError::Parse("no songs in response"));
    };

    let builder = parse_songs(&songs, builder)?;

    Ok(builder)
}

fn parse_songs(songs: &[RawSong], mut builder: AlbumBuilder) -> Result<AlbumBuilder, ExtractionError> {
    for song in songs {
        let raw_name = normalize_name(&song.song_name);
        let name = Name::new(&raw_name, LOCALE, true, true);

        let position = parse_position(&song.track_no)?;
        let duration = song.play_time;

        let mut song = Song::new(position, duration);
        song.add_name(name);

        builder = builder.add_song(song);
    }

    Ok(builder)
}

fn parse_album_id(url: &Url) -> Result<String, ExtractionError> {
    url.query_pairs()
        .find(|&(ref k, _)| k == "albumId")
        .and_then(|(_, v)| Some(v.into_owned()))
        .ok_or(ExtractionError::Url("missing query param `albumId`"))
}

fn parse_album_kind(s: &str) -> Result<AlbumKind, ExtractionError> {
    match s {
        // "OST" is not guaranteed, but is very likely, to be a single.
        "싱글" | "OST" => Ok(AlbumKind::Single),
        "EP" => Ok(AlbumKind::Ep),
        "정규" => Ok(AlbumKind::Lp),
        _ => Err(ExtractionError::Parse("album kind"))
    }
}

fn parse_artwork_url(s: &str) -> String {
    let end = s.len() - 4;
    let segment = &s[..end];
    format!("https://static.melon.co.kr{}_org.jpg", segment)
}

fn parse_position(s: &str) -> Result<i32, ExtractionError> {
    s.parse().map_err(|_| ExtractionError::Parse("position"))
}

fn parse_release_date(s: &str) -> Result<String, ExtractionError> {
    NaiveDate::parse_from_str(s, "%Y%m%d")
        .map(|d| d.format("%F").to_string())
        .map_err(|_| ExtractionError::Parse("release date"))
}

fn normalize_name(name: &str) -> String {
    name.replace("`", "'")
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
    use std::fs::File;
    use std::io::{self, BufReader};
    use std::io::prelude::*;
    use std::path::Path;

    use url::Url;

    use models::{AlbumKind, Name};
    use super::*;

    fn read_file<P>(pathname: P) -> io::Result<String> where P: AsRef<Path> {
        let file = File::open(pathname)?;
        let mut reader = BufReader::new(file);
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        Ok(data)
    }

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
        let html = read_file("test/fixtures/melon-10123637.html").unwrap();
        let json = read_file("test/fixtures/melon-10123637.json").unwrap();

        let album = parse("10123637", &html, &json).unwrap();

        assert_eq!(album.kind, AlbumKind::Single);
        assert_eq!(album.country, "KR");
        assert_eq!(album.released_on, "2017-12-28");
        assert_eq!(
            album.artwork_url,
            Some(String::from("https://static.melon.co.kr/cm/album/images/101/23/637/10123637_org.jpg")),
        );
        assert_eq!(album.url, "http://www.melon.com/album/detail.htm?albumId=10123637");

        assert_eq!(album.names.len(), 1);
        assert_eq!(&album.names[0], &Name::new("Chuu", "ko", true, true));

        assert_eq!(album.songs.len(), 2);

        let song = &album.songs[0];
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 195);
        assert_eq!(song.names.len(), 1);
        assert_eq!(&song.names[0], &Name::new("Heart Attack (츄)", "ko", true, true));

        let song = &album.songs[1];
        assert_eq!(song.position, 2);
        assert_eq!(song.duration, 197);
        assert_eq!(song.names.len(), 1);
        assert_eq!(&song.names[0], &Name::new("Girl's Talk (이브, 츄)", "ko", true, true));
    }

    #[test]
    fn test_parse_html_with_empty_document() {
        let builder = AlbumBuilder::new();
        assert!(parse_json("<html />", builder).is_err());
    }

    #[test]
    fn test_parse_json_with_empty_root() {
        let builder = AlbumBuilder::new();
        assert!(parse_json("{}", builder).is_err());
    }

    #[test]
    fn test_parse_album_id() {
        let url = Url::parse("http://www.melon.com/album/detail.htm?albumId=10141232").unwrap();
        assert_eq!(parse_album_id(&url), Ok(String::from("10141232")));

        let url = Url::parse("http://www.melon.com/album/detail.html").unwrap();
        assert!(parse_album_id(&url).is_err());
    }

    #[test]
    fn test_parse_album_kind() {
        assert_eq!(parse_album_kind("싱글"), Ok(AlbumKind::Single));
        assert_eq!(parse_album_kind("EP"), Ok(AlbumKind::Ep));
        assert_eq!(parse_album_kind("정규"), Ok(AlbumKind::Lp));
        assert_eq!(parse_album_kind("OST"), Ok(AlbumKind::Single));

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
        assert_eq!(parse_release_date("20171228"), Ok(String::from("2017-12-28")));
        assert!(parse_release_date("2017").is_err());
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Girl`s Talk (이브, 츄)"), "Girl's Talk (이브, 츄)");

        let actual = normalize_name("Love Don`t Hurt (Feat. Amber Of f(x))");
        let expected = "Love Don't Hurt (Feat. Amber of f(x))";
        assert_eq!(actual, expected);
    }
}
