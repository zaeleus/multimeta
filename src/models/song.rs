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

    pub fn new(position: i32, duration: i32) -> Song {
        Song {
            id: String::new(),
            position,
            duration,
            names: Vec::new(),
        }
    }

    pub fn default_name(&self) -> Option<String> {
        self.names
            .iter()
            .find(|&n| n.is_default)
            .map(|n| n.name.clone())
    }

    pub fn add_name(&mut self, name: Name) {
        self.names.push(name);
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
    use super::Song;
    use crate::models::Name;

    #[test]
    fn test_new() {
        let song = Song::new(1, 195);
        assert!(song.id.is_empty());
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 195);
        assert!(song.names.is_empty());
    }

    #[test]
    fn test_id() {
        let mut song = Song::new(1, 195);
        song.id = String::from("dun-dun");
        assert_eq!(song.id(), "dun-dun");
    }

    #[test]
    fn test_default_name() {
        let mut song = Song::new(1, 225);
        song.add_name(Name::new("꿈꾸는 마음으로", "ko", true, false));
        song.add_name(Name::new("Kkumkkuneun Maeumeuro", "ko-Latn", false, true));
        song.add_name(Name::new("Dreams Come True", "en", false, false));
        assert_eq!(
            song.default_name(),
            Some(String::from("Kkumkkuneun Maeumeuro"))
        );
    }

    #[test]
    fn test_add_name() {
        let mut song = Song::new(1, 195);
        let name = Name::new("Heart Attack (츄)", "ko", true, true);
        song.add_name(name);
        assert_eq!(song.names.len(), 1);
    }
}
