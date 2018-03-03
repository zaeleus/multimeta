use chrono::NaiveDate;
use reqwest;
use select::document::Document;
use select::predicate::{self, And, Attr};
use serde_json;
use url::Url;

use extractors::{ExtractionError, Extractor};
use models::{Album, AlbumBuilder, AlbumKind, Name, Song};

static HOST: &'static str = "mora.jp";

static HTML_BASE_URL: &'static str = "http://mora.jp/package";
static JSON_BASE_URL: &'static str = "http://cf.mora.jp/contents/package";
static JSON_FILENAME: &'static str = "packageMeta.json";

static COUNTRY: &'static str = "JP";
static LOCALE: &'static str = "ja";

pub struct MoraExtractor {
    album_id: String,
}

impl MoraExtractor {
    pub fn matches(url: &Url) -> bool {
        url.host_str().map(|h| h == HOST).unwrap_or(false)
    }

    pub fn new(url: &Url) -> Result<MoraExtractor, ExtractionError> {
        Ok(MoraExtractor {
            album_id: parse_album_id(url)?,
        })
    }

    fn fetch_html(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/", HTML_BASE_URL, self.album_id);
        fetch(&url)
    }
}

impl Extractor for MoraExtractor {
    fn extract(&self) -> Result<Album, ExtractionError> {
        let html = self.fetch_html().map_err(|_| ExtractionError::Fetch)?;

        let arguments = parse_html(&html)?;
        let json_endpoint = build_json_endpoint(
            &arguments.mount_point,
            &arguments.label_id,
            &arguments.material_no,
        );

        let json = fetch(&json_endpoint).map_err(|_| ExtractionError::Fetch)?;

        parse(&self.album_id, &json)
    }
}

fn fetch(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).and_then(|mut r| r.text())
}

fn parse(album_id: &str, json: &str) -> Result<Album, ExtractionError> {
    let builder = AlbumBuilder::new()
        .set_country(COUNTRY)
        .set_url(&format!("{}/{}/", HTML_BASE_URL, album_id));

    let builder = parse_json(json, builder)?;

    Ok(builder.build())
}

fn parse_html(html: &str) -> Result<Arguments, ExtractionError> {
    Document::from(html)
        .find(And(predicate::Name("meta"), Attr("name", "msApplication-Arguments")))
        .next()
        .and_then(|n| n.attr("content"))
        .map(|content| content.replace("&quot;", "\""))
        .and_then(|data| serde_json::from_str(&data).ok())
        .ok_or(ExtractionError::Parse("arguments"))
}

fn parse_json(json: &str, builder: AlbumBuilder) -> Result<AlbumBuilder, ExtractionError> {
    let root: Root = serde_json::from_str(json).map_err(|_| {
        ExtractionError::Parse("malformed JSON")
    })?;

    let songs = &root.track_list;

    let kind = guess_album_kind(songs.len());
    let name = Name::new(&root.title, LOCALE, true, true);

    let builder = builder
        .set_kind(kind)
        .set_released_on(&parse_release_date(&root.start_date)?)
        .add_name(name);

    let builder = parse_songs(songs, builder)?;

    Ok(builder)
}

fn parse_songs(songs: &[RawSong], mut builder: AlbumBuilder) -> Result<AlbumBuilder, ExtractionError> {
    for song in songs {
        let name = Name::new(&song.title, LOCALE, true, true);

        let mut song = Song::new(song.track_no, song.duration);
        song.add_name(name);

        builder = builder.add_song(song);
    }

    Ok(builder)
}

fn parse_album_id(url: &Url) -> Result<String, ExtractionError> {
    let pieces: Vec<&str> = url.path()
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();

    if pieces.len() < 2 {
        return Err(ExtractionError::Url("missing album ID in path"));
    }

    Ok(pieces[pieces.len() - 2..].join("/"))
}

fn parse_release_date(s: &str) -> Result<String, ExtractionError> {
    NaiveDate::parse_from_str(s, "%Y/%m/%d %H:%M:%S")
        .map(|d| d.format("%F").to_string())
        .map_err(|_| ExtractionError::Parse("release date"))
}

fn build_json_endpoint(mount_point: &str, label_id: &str, package_id: &str) -> String {
    let id = format!("{:0>10}", package_id);
    let (a, b, c) = (&id[0..4], &id[4..7], &id[7..10]);
    format!("{}/{}/{}/{}/{}/{}/{}", JSON_BASE_URL, mount_point, label_id, a, b, c, JSON_FILENAME)
}

// Guess the album kind based on the number of tracks.
fn guess_album_kind(n: usize) -> AlbumKind {
    if n <= 4 {
        AlbumKind::Single
    } else if n <= 6 {
        AlbumKind::Ep
    } else {
        AlbumKind::Lp
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Arguments {
  mount_point: String,
  label_id: String,
  material_no: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    start_date: String,
    title: String,
    track_list: Vec<RawSong>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSong {
    duration: i32,
    title: String,
    track_no: i32,
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_matches() {
        let url = Url::parse("http://mora.jp/package/43000001/4547366347050/").unwrap();
        assert!(MoraExtractor::matches(&url));

        let url = Url::parse("http://mora.jp/index_j").unwrap();
        assert!(MoraExtractor::matches(&url));

        let url = Url::parse("http://mora.jp/").unwrap();
        assert!(MoraExtractor::matches(&url));

        let url = Url::parse("https://www.google.com/").unwrap();
        assert!(!MoraExtractor::matches(&url))
    }

    #[test]
    fn test_parse() {
        let json = include_str!("../../test/fixtures/mora-43000001-4547366347050.json");
        let album = parse("43000001/4547366347050", json).unwrap();

        assert_eq!(album.kind, AlbumKind::Lp);
        assert_eq!(album.country, "JP");
        assert_eq!(album.released_on, "2018-02-12");
        assert!(album.artwork_url.is_none());
        assert_eq!(album.url, "http://mora.jp/package/43000001/4547366347050/");

        assert_eq!(album.names.len(), 1);
        assert_eq!(&album.names[0], &Name::new("HONEY", "ja", true, true));

        assert_eq!(album.songs.len(), 10);

        let song = &album.songs[0];
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 210);
        assert_eq!(song.names.len(), 1);
        assert_eq!(&song.names[0], &Name::new("プラットホームシンドローム", "ja", true, true));
    }

    #[test]
    fn test_parse_html() {
        let html = include_str!("../../test/fixtures/mora-43000001-4547366347050.html");
        let arguments = parse_html(html).unwrap();
        assert_eq!(arguments.mount_point, "0000");
        assert_eq!(arguments.label_id, "00000068");
        assert_eq!(arguments.material_no, "11174315");

        assert!(parse_html("<html />").is_err());
    }

    #[test]
    fn test_parse_json_with_empty_root() {
        let builder = AlbumBuilder::new();
        assert!(parse_json("{}", builder).is_err());
    }

    #[test]
    fn test_parse_album_id() {
        let url = Url::parse("http://mora.jp/package/43000001/4547366347050/").unwrap();
        assert_eq!(parse_album_id(&url), Ok(String::from("43000001/4547366347050")));

        let url = Url::parse("http://mora.jp/index_j").unwrap();
        assert!(parse_album_id(&url).is_err());
    }

    #[test]
    fn test_parse_release_date() {
        assert_eq!(parse_release_date("2018/02/12 00:00:00"), Ok(String::from("2018-02-12")));
        assert!(parse_release_date("2018").is_err());
    }

    #[test]
    fn test_build_json_endpoint() {
        let actual = build_json_endpoint("0000", "00000068", "11174315");
        let expected = "http://cf.mora.jp/contents/package/0000/00000068/0011/174/315/packageMeta.json";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_guess_album_kind() {
        assert_eq!(guess_album_kind(1), AlbumKind::Single);
        assert_eq!(guess_album_kind(2), AlbumKind::Single);
        assert_eq!(guess_album_kind(3), AlbumKind::Single);
        assert_eq!(guess_album_kind(4), AlbumKind::Single);

        assert_eq!(guess_album_kind(5), AlbumKind::Ep);
        assert_eq!(guess_album_kind(6), AlbumKind::Ep);

        assert_eq!(guess_album_kind(7), AlbumKind::Lp);
        assert_eq!(guess_album_kind(8), AlbumKind::Lp);
        assert_eq!(guess_album_kind(9), AlbumKind::Lp);
        assert_eq!(guess_album_kind(10), AlbumKind::Lp);
        assert_eq!(guess_album_kind(11), AlbumKind::Lp);
    }
}
