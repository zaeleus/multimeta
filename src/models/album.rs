use std::fmt;

use serde::Serialize;

use crate::{
    editor::AlbumInput,
    models::{Name, Song},
    util::inflector::parameterize,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Single,
    Ep,
    Lp,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Kind::Single => write!(f, "single"),
            Kind::Ep => write!(f, "ep"),
            Kind::Lp => write!(f, "lp"),
        }
    }
}

#[derive(Serialize)]
pub struct Album {
    pub id: String,

    pub kind: Kind,
    pub country: String,
    pub released_on: String,
    pub artwork_url: Option<String>,
    pub url: String,

    pub names: Vec<Name>,
    pub songs: Vec<Song>,
}

impl Album {
    pub fn id(&self) -> &str {
        &self.id
    }

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
            id: input.id,

            kind: input.kind,
            country: input.country,
            released_on: input.released_on,
            artwork_url: input.artwork_url,
            url: input.url,

            names: input
                .names
                .into_iter()
                .filter(|n| !n.delete)
                .map(Name::from)
                .collect(),
            songs: input.songs.into_iter().map(Song::from).collect(),
        }
    }
}

#[derive(Default)]
pub struct AlbumBuilder {
    pub id: Option<String>,

    pub kind: Option<Kind>,
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

    pub fn set_id(mut self, id: &str) -> AlbumBuilder {
        self.id = Some(id.to_owned());
        self
    }

    pub fn set_kind(mut self, kind: Kind) -> AlbumBuilder {
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
        let id = self
            .id
            .clone()
            .or_else(|| {
                self.names
                    .iter()
                    .find(|n| n.is_default)
                    .map(|n| parameterize(&n.name))
            })
            .expect("missing id");

        Album {
            id,

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
    use super::*;

    fn build_album() -> Album {
        AlbumBuilder::new()
            .set_id("from-wjsn")
            .set_kind(Kind::Single)
            .set_country("KR")
            .set_released_on("2017-01-04")
            .set_artwork_url("http://localhost/artwork.jpg")
            .set_url("http://localhost/albums/1")
            .add_name(Name::new("From. 우주소녀", "ko", true, false))
            .add_name(Name::new("From. WJSN", "en", false, true))
            .build()
    }

    #[test]
    fn test_fmt() {
        assert_eq!(Kind::Single.to_string(), "single");
        assert_eq!(Kind::Ep.to_string(), "ep");
        assert_eq!(Kind::Lp.to_string(), "lp");
    }

    #[test]
    fn test_id() {
        let album = build_album();
        assert_eq!(album.id(), "from-wjsn");
    }

    #[test]
    fn test_default_name() {
        let album = build_album();
        assert_eq!(album.default_name(), Some(String::from("From. WJSN")));
    }
}
