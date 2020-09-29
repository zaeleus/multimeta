mod builder;
mod kind;

pub use self::{builder::Builder, kind::Kind};

use serde::Serialize;

use crate::{
    editor::AlbumInput,
    models::{Name, Song},
};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_album() -> Album {
        Builder::new()
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
