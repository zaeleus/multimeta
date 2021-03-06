use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use log::{info, log_enabled, Level};

use crate::{
    models::Album,
    renderer::Renderer,
    util::{
        http::{self, Downloader},
        jpeg,
    },
};

pub struct Writer {
    dst_prefix: PathBuf,
}

impl Writer {
    pub fn new<P>(dst_prefix: P) -> Writer
    where
        P: AsRef<Path>,
    {
        Writer {
            dst_prefix: dst_prefix.as_ref().into(),
        }
    }

    pub fn write_templates(
        &self,
        renderer: &Renderer,
        artist_id: &str,
        album: &Album,
    ) -> io::Result<()> {
        self.write_album(renderer, artist_id, album)?;
        self.write_songs(renderer, artist_id, album)?;
        self.write_tracklist(renderer, artist_id, album)?;
        Ok(())
    }

    fn write_album(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        let dst_prefix = self.dst_prefix.join("albums").join(artist_id);

        fs::create_dir_all(&dst_prefix)?;

        let basename = album.id();
        let dst = dst_prefix.join(format!("{}.toml", basename));

        let result = renderer.render_album(artist_id, album);

        write_file(&dst, result.as_bytes())
    }

    fn write_songs(&self, renderer: &Renderer, artist_id: &str, album: &Album) -> io::Result<()> {
        let dst_prefix = self.dst_prefix.join("songs").join(artist_id);

        fs::create_dir_all(&dst_prefix)?;

        for song in &album.songs {
            let basename = song.id();
            let dst = dst_prefix.join(format!("{}.toml", basename));

            let result = renderer.render_song(song);

            write_file(&dst, result.as_bytes())?;
        }

        Ok(())
    }

    fn write_tracklist(
        &self,
        renderer: &Renderer,
        artist_id: &str,
        album: &Album,
    ) -> io::Result<()> {
        let dst_prefix = self
            .dst_prefix
            .join("tracklists")
            .join(artist_id)
            .join(album.id())
            .join("default");

        fs::create_dir_all(&dst_prefix)?;

        let dst = dst_prefix.join("digital1.toml");

        let result = renderer.render_tracklist(artist_id, album);

        write_file(&dst, result.as_bytes())
    }

    pub fn write_artwork(&self, artist_id: &str, album: &Album) -> io::Result<()> {
        let mut dst_prefix = self
            .dst_prefix
            .join("-attachments")
            .join("albums")
            .join(artist_id)
            .join(album.id())
            .join("-original");

        fs::create_dir_all(&dst_prefix)?;

        let original_dst = dst_prefix.join("default.jpg");

        dst_prefix.pop();
        let final_dst = dst_prefix.join("default.jpg");

        if let Some(ref artwork_url) = album.artwork_url {
            let downloader = Downloader::new();

            downloader
                .save(artwork_url, &original_dst)
                .map_err(|e| match e {
                    http::Error::Io(inner) => inner,
                    http::Error::RequestFailed => {
                        io::Error::new(io::ErrorKind::Other, "request failed")
                    }
                    http::Error::EmptyBody => io::Error::new(io::ErrorKind::Other, "empty body"),
                })?;

            optimize(&original_dst, &final_dst)?;
        }

        Ok(())
    }
}

fn write_file<P>(pathname: P, data: &[u8]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = File::create(pathname)?;
    file.write_all(data)
}

fn optimize<P, Q>(src: P, dst: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let versions = jpeg::optimize(src)?;

    if log_enabled!(Level::Info) {
        let info = versions
            .iter()
            .map(|v| format!("{} ({} KiB)", v.name, v.filesize / 1024))
            .collect::<Vec<String>>()
            .join(", ");
        info!("{} version(s): {}", versions.len(), info);
    }

    match versions.iter().min_by_key(|v| v.filesize) {
        Some(v) => fs::rename(&v.src, dst),
        None => Err(io::Error::new(io::ErrorKind::Other, "invalid versions")),
    }
}
