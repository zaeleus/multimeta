use serde::Serialize;

use crate::{editor::SongInput, models::Name, util::inflector::parameterize};

#[derive(Debug, Serialize)]
pub struct Song {
    pub id: String,

    pub position: i32,
    pub duration: i32,

    pub names: Vec<Name>,
}

impl Song {
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

impl From<SongInput> for Song {
    fn from(input: SongInput) -> Song {
        Song {
            id: input.id,

            position: input.position,
            duration: input.duration,

            names: input
                .names
                .into_iter()
                .filter(|n| !n.delete)
                .map(Name::from)
                .collect(),
        }
    }
}

#[derive(Default)]
pub struct SongBuilder {
    pub id: Option<String>,

    pub position: Option<i32>,
    pub duration: Option<i32>,

    pub names: Vec<Name>,
}

impl SongBuilder {
    pub fn new() -> SongBuilder {
        SongBuilder::default()
    }

    pub fn set_id(mut self, id: &str) -> SongBuilder {
        self.id = Some(id.to_owned());
        self
    }

    pub fn set_position(mut self, position: i32) -> SongBuilder {
        self.position = Some(position);
        self
    }

    pub fn set_duration(mut self, duration: i32) -> SongBuilder {
        self.duration = Some(duration);
        self
    }

    pub fn add_name(mut self, name: Name) -> SongBuilder {
        self.names.push(name);
        self
    }

    pub fn build(self) -> Song {
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

        Song {
            id,

            position: self.position.expect("missing position"),
            duration: self.duration.expect("missing duration"),

            names: self.names,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_song() -> Song {
        SongBuilder::new()
            .set_position(1)
            .set_duration(225)
            .add_name(Name::new("꿈꾸는 마음으로", "ko", true, false))
            .add_name(Name::new("Kkumkkuneun Maeumeuro", "ko-Latn", false, true))
            .add_name(Name::new("Dreams Come True", "en", false, false))
            .build()
    }

    #[test]
    fn test_id() {
        let song = build_song();
        assert_eq!(song.id(), "kkumkkuneun-maeumeuro");
    }

    #[test]
    fn test_default_name() {
        let song = build_song();

        assert_eq!(
            song.default_name(),
            Some(String::from("Kkumkkuneun Maeumeuro"))
        );
    }
}
