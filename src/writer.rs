use std::io;
use std::io::prelude::*;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use models::Album;
use renderer::Renderer;
use util::parameterize;

pub struct Writer {
    output_dir: String,
}

impl Writer {
    pub fn new(output_dir: &str) -> Writer {
        Writer { output_dir: output_dir.to_owned() }
    }

    pub fn write(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        self.write_album(renderer, artist_id, album)?;
        self.write_songs(renderer, artist_id, album)?;
        self.write_tracklist(renderer, artist_id, album)?;
        Ok(())
    }

    fn write_album(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        let mut albums_dir = PathBuf::from(&self.output_dir);
        albums_dir.push("albums");
        albums_dir.push(artist_id);

        fs::create_dir_all(&albums_dir)?;

        let album_name = album.default_name().expect("no default album name");
        let basename = parameterize(&album_name);
        let mut pathname = albums_dir.clone();
        pathname.push(&format!("{}.toml", basename));

        let result = renderer.render_album(artist_id, album);

        write_file(&pathname, &result)
    }

    fn write_songs(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        let mut songs_dir = PathBuf::from(&self.output_dir);
        songs_dir.push("songs");
        songs_dir.push(artist_id);

        fs::create_dir_all(&songs_dir)?;

        for song in &album.songs {
            let song_name = song.default_name().expect("no default name");
            let basename = parameterize(&song_name);
            let mut pathname = songs_dir.clone();
            pathname.push(&format!("{}.toml", basename));

            let result = renderer.render_song(song);

            write_file(&pathname, &result)?;
        }

        Ok(())
    }

    fn write_tracklist(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        let album_name = album.default_name().unwrap();

        let mut tracklist_dir = PathBuf::from(&self.output_dir);
        tracklist_dir.push("tracklists");
        tracklist_dir.push(artist_id);
        tracklist_dir.push(&parameterize(&album_name));
        tracklist_dir.push("default");

        fs::create_dir_all(&tracklist_dir)?;

        let mut pathname = tracklist_dir.clone();
        pathname.push("digital1.toml");

        let result = renderer.render_tracklist(artist_id, album);

        write_file(&pathname, &result)
    }
}

fn write_file<P>(pathname: P, data: &str) -> io::Result<()> where P: AsRef<Path> {
    let mut file = File::create(pathname)?;
    file.write_all(data.as_bytes())
}
