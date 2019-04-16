use std::fmt;

use serde::Serialize;

use crate::editor::AlbumInput;
use crate::models::{Name, Song};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlbumKind {
    Single,
    Ep,
    Lp,
}

impl fmt::Display for AlbumKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AlbumKind::Single => write!(f, "single"),
            AlbumKind::Ep => write!(f, "ep"),
            AlbumKind::Lp => write!(f, "lp"),
        }
    }
}

#[derive(Serialize)]
pub struct Album {
    pub kind: AlbumKind,
    pub country: String,
    pub released_on: String,
    pub artwork_url: Option<String>,
    pub url: String,

    pub names: Vec<Name>,
    pub songs: Vec<Song>,
}

impl Album {
    pub fn default_name(&self) -> Option<String> {
        self.names
            .iter()
            .find(|&n| n.is_default)
            .map(|n| n.name.clone())
    }
}

impl From<AlbumInput> for Album {
    fn from(input: AlbumInput) -> Album {
        Album {
            kind: input.kind,
            country: input.country,
            released_on: input.released_on,
            artwork_url: input.artwork_url,
            url: input.url,

            names: input
                .names
                .into_iter()
                .filter(|n| !n._delete)
                .map(Name::from)
                .collect(),
            songs: input.songs.into_iter().map(Song::from).collect(),
        }
    }
}

#[derive(Default)]
pub struct AlbumBuilder {
    pub kind: Option<AlbumKind>,
    pub country: Option<String>,
    pub released_on: Option<String>,
    pub artwork_url: Option<String>,
    pub url: Option<String>,

    pub names: Vec<Name>,
    pub songs: Vec<Song>,
}

impl AlbumBuilder {
    pub fn new() -> AlbumBuilder {
        AlbumBuilder::default()
    }

    pub fn set_kind(mut self, kind: AlbumKind) -> AlbumBuilder {
        self.kind = Some(kind);
        self
    }

    pub fn set_country(mut self, country: &str) -> AlbumBuilder {
        self.country = Some(country.to_owned());
        self
    }

    pub fn set_released_on(mut self, released_on: &str) -> AlbumBuilder {
        self.released_on = Some(released_on.to_owned());
        self
    }

    pub fn set_artwork_url(mut self, artwork_url: &str) -> AlbumBuilder {
        self.artwork_url = Some(artwork_url.to_owned());
        self
    }

    pub fn set_url(mut self, url: &str) -> AlbumBuilder {
        self.url = Some(url.to_owned());
        self
    }

    pub fn add_name(mut self, name: Name) -> AlbumBuilder {
        self.names.push(name);
        self
    }

    pub fn add_song(mut self, song: Song) -> AlbumBuilder {
        self.songs.push(song);
        self
    }

    pub fn build(self) -> Album {
        Album {
            kind: self.kind.expect("missing kind"),
            country: self.country.expect("missing country"),
            released_on: self.released_on.expect("missing released on"),
            artwork_url: self.artwork_url,
            url: self.url.expect("missing url"),

            names: self.names,
            songs: self.songs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlbumBuilder, AlbumKind};
    use crate::models::Name;

    #[test]
    fn test_fmt() {
        assert_eq!(AlbumKind::Single.to_string(), "single");
        assert_eq!(AlbumKind::Ep.to_string(), "ep");
        assert_eq!(AlbumKind::Lp.to_string(), "lp");
    }

    #[test]
    fn test_default_name() {
        let album = AlbumBuilder::new()
            .set_kind(AlbumKind::Single)
            .set_country("KR")
            .set_released_on("2017-01-04")
            .set_artwork_url("http://localhost/artwork.jpg")
            .set_url("http://localhost/albums/1")
            .add_name(Name::new("From. 우주소녀", "ko", true, false))
            .add_name(Name::new("From. WJSN", "en", false, true))
            .build();

        assert_eq!(album.default_name(), Some(String::from("From. WJSN")));
    }
}
