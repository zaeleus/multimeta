use std::io;
use std::io::prelude::*;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use log::Level;

use models::Album;
use renderer::Renderer;
use util::http::{self, Downloader};
use util::inflector::parameterize;
use util::jpeg;

pub struct Writer {
    output_dir: String,
}

impl Writer {
    pub fn new(output_dir: &str) -> Writer {
        Writer { output_dir: output_dir.to_owned() }
    }

    pub fn write_templates(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
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

    pub fn write_artwork(&self, artist_id: &str, album: &Album) -> io::Result<()> {
        let album_name = album.default_name().unwrap();

        let mut artwork_dir = PathBuf::from(&self.output_dir);
        artwork_dir.push("-attachments");
        artwork_dir.push("albums");
        artwork_dir.push(artist_id);
        artwork_dir.push(&parameterize(&album_name));
        artwork_dir.push("-original");

        fs::create_dir_all(&artwork_dir)?;

        let mut original_pathname = artwork_dir.clone();
        original_pathname.push("default.jpg");

        artwork_dir.pop();

        let mut final_pathname = artwork_dir.clone();
        final_pathname.push("default.jpg");

        if let Some(ref artwork_url) = album.artwork_url {
            let downloader = Downloader::new();

            downloader.save(artwork_url, &original_pathname).map_err(|e| {
                match e {
                    http::Error::Io(inner) => inner,
                    http::Error::RequestFailed => {
                        io::Error::new(io::ErrorKind::Other, "request failed")
                    },
                    http::Error::EmptyBody => {
                        io::Error::new(io::ErrorKind::Other, "empty body")
                    },
                }
            })?;

            optimize(&original_pathname, &final_pathname)?;
        }

        Ok(())
    }
}

fn write_file<P>(pathname: P, data: &str) -> io::Result<()> where P: AsRef<Path> {
    let mut file = File::create(pathname)?;
    file.write_all(data.as_bytes())
}

fn optimize<P, Q>(src: P, dst: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let versions = jpeg::optimize(src)?;

    if log_enabled!(Level::Info) {
        let info = versions.iter()
            .map(|v| format!("{} ({} KiB)", v.name, v.filesize / 1024))
            .collect::<Vec<String>>()
            .join(", ");
        info!("{} version(s): {}", versions.len(), info);
    }

    match versions.iter().min_by_key(|v| v.filesize) {
        Some(v) => fs::rename(&v.pathname, dst),
        None => Err(io::Error::new(io::ErrorKind::Other, "invalid versions")),
    }
}
