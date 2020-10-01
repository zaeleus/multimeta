mod builder;

pub use self::builder::Builder;

use serde::Serialize;

use crate::{editor::SongInput, models::Name};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_song() -> Song {
        Builder::new()
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
