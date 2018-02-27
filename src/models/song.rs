use models::Name;

#[derive(Debug)]
pub struct Song {
    pub position: i32,
    pub duration: i32,

    pub names: Vec<Name>,
}

impl Song {
    pub fn new(position: i32, duration: i32) -> Song {
        Song { position: position, duration: duration, names: Vec::new() }
    }

    pub fn add_name(&mut self, name: Name) {
        self.names.push(name);
    }
}

#[cfg(test)]
mod tests {
    use models::Name;
    use super::Song;

    #[test]
    fn test_new() {
        let song = Song::new(1, 195);
        assert_eq!(song.position, 1);
        assert_eq!(song.duration, 195);
        assert!(song.names.is_empty());
    }

    #[test]
    fn test_add_name() {
        let mut song = Song::new(1, 195);
        let name = Name::new("Heart Attack (츄)", "ko", true, true);
        song.add_name(name);
        assert_eq!(song.names.len(), 1);
    }
}
