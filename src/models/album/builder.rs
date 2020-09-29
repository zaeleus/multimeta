use crate::{
    models::{Name, Song},
    util::inflector::parameterize,
};

use super::{Album, Kind};

#[derive(Default)]
pub struct Builder {
    pub id: Option<String>,

    pub kind: Option<Kind>,
    pub country: Option<String>,
    pub released_on: Option<String>,
    pub artwork_url: Option<String>,
    pub url: Option<String>,

    pub names: Vec<Name>,
    pub songs: Vec<Song>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_owned());
        self
    }

    pub fn set_kind(mut self, kind: Kind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn set_country(mut self, country: &str) -> Self {
        self.country = Some(country.to_owned());
        self
    }

    pub fn set_released_on(mut self, released_on: &str) -> Self {
        self.released_on = Some(released_on.to_owned());
        self
    }

    pub fn set_artwork_url(mut self, artwork_url: &str) -> Self {
        self.artwork_url = Some(artwork_url.to_owned());
        self
    }

    pub fn set_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_owned());
        self
    }

    pub fn add_name(mut self, name: Name) -> Self {
        self.names.push(name);
        self
    }

    pub fn add_song(mut self, song: Song) -> Self {
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
