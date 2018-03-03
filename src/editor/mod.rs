use self::readline::readline;

mod readline;

use models::{Album, AlbumKind, Name, Song};
use util::format_duration;

pub struct AlbumInput {
    pub kind: AlbumKind,
    pub country: String,
    pub released_on: String,
    pub artwork_url: Option<String>,
    pub url: String,

    pub names: Vec<NameInput>,
    pub songs: Vec<SongInput>,
}

impl<'a> From<&'a Album> for AlbumInput {
    fn from(album: &'a Album) -> AlbumInput {
        AlbumInput {
            kind: album.kind,
            country: album.country.clone(),
            released_on: album.released_on.clone(),
            artwork_url: album.artwork_url.clone(),
            url: album.url.clone(),

            names: album.names.iter().map(NameInput::from).collect(),
            songs: album.songs.iter().map(SongInput::from).collect(),
        }
    }
}

#[derive(Default)]
pub struct NameInput {
    pub name: String,
    pub locale: String,
    pub is_original: bool,
    pub is_default: bool,
}

impl<'a> From<&'a Name> for NameInput {
    fn from(name: &'a Name) -> NameInput {
        NameInput {
            name: name.name.clone(),
            locale: name.locale.clone(),
            is_original: name.is_original,
            is_default: name.is_default,
        }
    }
}

pub struct SongInput {
    pub position: i32,
    pub duration: i32,

    pub names: Vec<NameInput>,
}

impl<'a> From<&'a Song> for SongInput {
    fn from(song: &'a Song) -> SongInput {
        SongInput {
            position: song.position,
            duration: song.duration,

            names: song.names.iter().map(NameInput::from).collect(),
        }
    }
}

pub fn edit(album: &Album) -> Album {
    let mut form = prepare(album);

    loop {
        edit_album(&mut form);

        if let Ok(input) = readline("> Commit? [Y/n] ") {
            if input.is_empty() || input == "y" {
                break;
            }
        }
    }

    commit(form)
}

fn prepare(album: &Album) -> AlbumInput {
    AlbumInput::from(album)
}

fn commit(input: AlbumInput) -> Album {
    Album::from(input)
}

fn edit_album(album: &mut AlbumInput) {
    println!("kind: {}", album.kind);
    println!("released on: {}", album.released_on);

    edit_names(&mut album.names);
    edit_songs(&mut album.songs);
}

fn edit_songs(songs: &mut [SongInput]) {
    for song in songs {
        println!("position: {}", song.position);
        println!("duration: {}", format_duration(song.duration));

        edit_names(&mut song.names);
    }
}

fn edit_names(names: &mut Vec<NameInput>) {
    loop {
        println!("names:");

        for (i, name) in names.iter().enumerate() {
            println!(
                "  {}. {} (locale: {}, original: {}, default: {})",
                i, name.name, name.locale, name.is_original, name.is_default,
            );
        }

        println!();

        if let Ok(input) = readline("> Edit name? [a/e/N] ") {
            match input.as_ref() {
                "a" => {
                    add_name(names);
                    let i = names.len() - 1;
                    update_name_flags(names, i);
                },
                "e" => {
                    let i = prompt_index();

                    if i < names.len() {
                        edit_name(&mut names[i]);
                        update_name_flags(names, i);
                    }
                },
                "n" | "" => break,
                _ => {},
            }
        }

        println!();
    }
}

fn add_name(names: &mut Vec<NameInput>) {
    let mut name = NameInput::default();
    edit_name(&mut name);
    names.push(name);
}

fn edit_name(name: &mut NameInput) {
    if let Ok(raw_name) = readline(&format!("  name [{}]: ", name.name)) {
        if !raw_name.is_empty() {
            name.name = raw_name;
        }
    }

    if let Ok(locale) = readline(&format!("  locale [{}]: ", name.locale)) {
        if !locale.is_empty() {
            name.locale = locale;
        }
    }

    if let Ok(is_original) = readline(&format!("  original [{}]: ", name.is_original)) {
        if !is_original.is_empty() {
            name.is_original = parse_boolean(&is_original);
        }
    }

    if let Ok(is_default) = readline(&format!("  default [{}]: ", name.is_default)) {
        if !is_default.is_empty() {
            name.is_default = parse_boolean(&is_default);
        }
    }
}

fn prompt_index() -> usize {
    readline("> Index: ").ok().and_then(|i| i.parse().ok()).unwrap_or(0)
}

fn update_name_flags(names: &mut [NameInput], i: usize) {
    let (is_original, is_default) = {
        let name = &names[i];
        (name.is_original, name.is_default)
    };

    for (j, name) in names.iter_mut().enumerate() {
        if i == j { continue; }

        if is_original {
            name.is_original = false;
        }

        if is_default {
            name.is_default = false;
        }
    }
}

fn parse_boolean(s: &str) -> bool {
    s == "true" || s == "t"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_name_flags() {
        let mut names = vec![
            NameInput { is_original: true, ..NameInput::default() },
            NameInput { is_default: true, ..NameInput::default() },
            NameInput { is_original: true, ..NameInput::default() },
        ];

        update_name_flags(&mut names, 2);

        assert!(!names[0].is_original);
        assert!(!names[0].is_default);
        assert!(!names[1].is_original);
        assert!(names[1].is_default);
        assert!(names[2].is_original);
        assert!(!names[2].is_default);
    }

    #[test]
    fn test_parse_boolean() {
        assert!(parse_boolean("true"));
        assert!(parse_boolean("t"));

        assert!(!parse_boolean("false"));
        assert!(!parse_boolean("f"));
        assert!(!parse_boolean("tru"));
        assert!(!parse_boolean("yes"));
        assert!(!parse_boolean("y"));
    }
}
