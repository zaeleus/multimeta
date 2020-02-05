use rustyline::{self, error::ReadlineError};

use crate::{
    models::{Album, AlbumKind, Name, Song},
    util::{
        format_duration,
        inflector::{parameterize, titleize},
    },
};

pub struct AlbumInput {
    pub id: String,

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
            id: album.id().into(),

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

    pub delete: bool,
}

impl<'a> From<&'a Name> for NameInput {
    fn from(name: &'a Name) -> NameInput {
        NameInput {
            name: name.name.clone(),
            locale: name.locale.clone(),
            is_original: name.is_original,
            is_default: name.is_default,

            delete: false,
        }
    }
}

pub struct SongInput {
    pub id: String,

    pub position: i32,
    pub duration: i32,

    pub names: Vec<NameInput>,
}

impl<'a> From<&'a Song> for SongInput {
    fn from(song: &'a Song) -> SongInput {
        SongInput {
            id: song.id().into(),

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

    edit_names(&mut album.id, &mut album.names);
    edit_songs(&mut album.songs);
}

fn edit_songs(songs: &mut [SongInput]) {
    for song in songs {
        println!("position: {}", song.position);
        println!("duration: {}", format_duration(song.duration));

        edit_names(&mut song.id, &mut song.names);
    }
}

fn edit_names(id: &mut String, names: &mut Vec<NameInput>) {
    loop {
        println!("id: {}", id);
        println!("names:");

        for (i, name) in names.iter().enumerate() {
            let status = if name.delete { "*" } else { "" };

            println!(
                "  {}{}. {} (locale: {}, original: {}, default: {})",
                i, status, name.name, name.locale, name.is_original, name.is_default,
            );
        }

        println!();

        if let Ok(input) = readline("> Edit name? [a/e/d/g/N] ") {
            match input.as_ref() {
                "a" => {
                    add_name(names);
                    let i = names.len() - 1;
                    update_name_flags(names, i);
                }
                "e" => {
                    let i = prompt_index();

                    if i < names.len() {
                        edit_name(&mut names[i]);
                        update_name_flags(names, i);
                    }
                }
                "d" => {
                    let i = prompt_index();

                    if i < names.len() {
                        delete_name(names, i);
                    }
                }
                "g" => {
                    if guess_name(names) {
                        let i = names.len() - 1;
                        update_name_flags(names, i);
                    }
                }
                "n" | "" => break,
                _ => {}
            }
        }

        *id = parameterize(&default_name(names).expect("missing default name"));

        println!();
    }
}

fn add_name(names: &mut Vec<NameInput>) {
    let mut name = NameInput::default();
    edit_name(&mut name);
    names.push(name);
}

fn edit_name(name: &mut NameInput) {
    let prompt = format!("  name [{}]: ", name.name);
    if let Ok(raw_name) = editline(&prompt, &name.name) {
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

fn delete_name(names: &mut Vec<NameInput>, i: usize) {
    let name = &mut names[i];
    name.delete = !name.delete;
}

/// Adds a new name based on some heuristics.
///
/// This currently only support Korean romanization.
///
/// It returns `true` if a new name is added and `false` if no conditions matched to add a new
/// name.
fn guess_name(names: &mut Vec<NameInput>) -> bool {
    let original_name = names
        .iter()
        .find(|n| n.is_original && n.locale == "ko")
        .map(|n| n.name.clone());

    if let Some(name) = original_name {
        let new_name = NameInput {
            name: titleize(&hangeul::romanize(&name)),
            locale: String::from("ko-Latn"),
            is_original: false,
            is_default: true,
            ..NameInput::default()
        };

        names.push(new_name);

        return true;
    }

    false
}

fn prompt_index() -> usize {
    readline("> Index: ")
        .ok()
        .and_then(|i| i.parse().ok())
        .unwrap_or(0)
}

fn update_name_flags(names: &mut [NameInput], i: usize) {
    let (is_original, is_default) = {
        let name = &names[i];
        (name.is_original, name.is_default)
    };

    for (j, name) in names.iter_mut().enumerate() {
        if i == j {
            continue;
        }

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

fn readline(prompt: &str) -> rustyline::Result<String> {
    let mut rl = rustyline::Editor::<()>::new();
    exit_on_interrupted(rl.readline(prompt))
}

fn editline(prompt: &str, text: &str) -> rustyline::Result<String> {
    let mut rl = rustyline::Editor::<()>::new();
    exit_on_interrupted(rl.readline_with_initial(prompt, (text, "")))
}

fn exit_on_interrupted(result: rustyline::Result<String>) -> rustyline::Result<String> {
    match result {
        Ok(s) => Ok(s),
        Err(e) => match e {
            // Errno 130 matches the behavior of readline.
            ReadlineError::Interrupted => std::process::exit(130),
            _ => Err(e),
        },
    }
}

fn default_name(names: &[NameInput]) -> Option<String> {
    names.iter().find(|n| n.is_default).map(|n| n.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_name_flags() {
        let mut names = vec![
            NameInput {
                is_original: true,
                ..NameInput::default()
            },
            NameInput {
                is_default: true,
                ..NameInput::default()
            },
            NameInput {
                is_original: true,
                ..NameInput::default()
            },
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
    fn test_delete_name() {
        let mut names = vec![NameInput::default()];

        delete_name(&mut names, 0);
        assert!(names[0].delete);

        delete_name(&mut names, 0);
        assert!(!names[0].delete);
    }

    #[test]
    fn test_guess_name() {
        let mut names = vec![NameInput {
            name: String::from("비밀이야"),
            locale: String::from("ko"),
            is_original: true,
            is_default: true,
            ..NameInput::default()
        }];

        guess_name(&mut names);

        assert_eq!(names.len(), 2);

        let name = &names[1];
        assert_eq!(name.name, "Bimiriya");
        assert_eq!(name.locale, "ko-Latn");
        assert!(!name.is_original);
        assert!(name.is_default);
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

    #[test]
    fn test_default_name() {
        let names = vec![
            NameInput {
                name: String::from("비밀이야"),
                ..NameInput::default()
            },
            NameInput {
                name: String::from("Bimiriya"),
                is_default: true,
                ..NameInput::default()
            },
        ];

        assert_eq!(default_name(&names), Some(String::from("Bimiriya")));

        let names = vec![];
        assert!(default_name(&names).is_none());
    }
}
