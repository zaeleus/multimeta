use crate::{models::Name, util::inflector::parameterize};

use super::Song;

#[derive(Default)]
pub struct Builder {
    pub id: Option<String>,

    pub position: Option<i32>,
    pub duration: Option<i32>,

    pub names: Vec<Name>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_owned());
        self
    }

    pub fn set_position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn set_duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn add_name(mut self, name: Name) -> Self {
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
