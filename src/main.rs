use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use clap::{crate_name, value_t, App, Arg};
use git_testament::{git_testament, render_testament};
use glob::glob;
use log::{log_enabled, warn, Level, LevelFilter};
use reqwest::Url;

use multimeta::{editor, extractors, renderer::Renderer, writer::Writer};

git_testament!(TESTAMENT);

// This is required to be `Fn(String) -> _` to be used as a clap validator.
#[allow(clippy::needless_pass_by_value)]
fn validate_output(s: String) -> Result<(), String> {
    let path = Path::new(&s);

    if path.is_dir() {
        Ok(())
    } else {
        Err(String::from("Not a directory"))
    }
}

fn get_artists<P>(output_dir: P) -> HashSet<String>
where
    P: AsRef<Path>,
{
    static KINDS: [&str; 2] = ["people", "groups"];

    let mut set = HashSet::new();

    // Resolve the absolute path since a leading current directory component
    // does not appear in the glob results.
    let output_dir = output_dir.as_ref().canonicalize().unwrap();

    for kind in &KINDS {
        let prefix = output_dir.join("artists").join(kind);
        let pattern = prefix.join("**/*.toml");
        let pattern = pattern.to_str().unwrap();

        let entries = glob(pattern)
            .expect("invalid glob pattern")
            .filter_map(Result::ok);

        for entry in entries {
            let path = entry
                .strip_prefix(&prefix)
                .map(|p| p.with_extension(""))
                .expect("entry must have prefix");

            let id = path.to_str().unwrap();
            set.insert(String::from(id));
        }
    }

    set
}

fn check_artist_id<P>(output_dir: P, artist_id: &str)
where
    P: AsRef<Path>,
{
    let artists = get_artists(output_dir);

    if !artists.contains(artist_id) {
        warn!("artist id '{}' does not exist", artist_id);
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(render_testament!(TESTAMENT).as_str())
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("DIR")
                .help("Set output directory")
                .default_value(".")
                .validator(validate_output),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Use verbose logging"),
        )
        .arg(
            Arg::with_name("artist-id")
                .help("The local artist ID")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("url")
                .help("The remote URL to scrape")
                .index(2)
                .required(true),
        )
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }

    let output_dir = value_t!(matches, "output", PathBuf).unwrap_or_else(|e| e.exit());

    let artist_id = matches.value_of("artist-id").unwrap();
    let url = value_t!(matches, "url", Url).unwrap_or_else(|e| e.exit());

    if log_enabled!(Level::Warn) {
        check_artist_id(&output_dir, &artist_id);
    }

    let album = extractors::factory(&url)
        .and_then(|e| e.extract())
        .map(|a| editor::edit(&a))
        .unwrap_or_else(|e| panic!("{:?}", e));

    let writer = Writer::new(&output_dir);

    if album.artwork_url.is_some() {
        if let Err(e) = writer.write_artwork(&artist_id, &album) {
            warn!("failed to download artwork ({:?})", e);
        }
    }

    let renderer = Renderer::new();
    writer
        .write_templates(&renderer, &artist_id, &album)
        .expect("write failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_artists() {
        let artists = get_artists("test/fixtures/fs");

        assert_eq!(artists.len(), 3);

        assert!(artists.contains("apink/chorong"));
        assert!(artists.contains("bol4"));
        assert!(artists.contains("i"));
    }
}
