use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

use log::info;
use pbr::{ProgressBar, Units};

const DEFAULT_BUF_SIZE: usize = 8192; // bytes

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    RequestFailed,
    EmptyBody,
}

pub struct Downloader {
    client: ureq::Agent,
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader::default()
    }

    pub fn save<P>(&self, url: &str, dst: P) -> Result<u64, Error>
    where
        P: AsRef<Path>,
    {
        info!("downloading {}", url);

        let file = File::create(dst).map_err(Error::Io)?;
        let mut writer = BufWriter::new(file);

        let response = self
            .client
            .get(url)
            .call()
            .map_err(|_| Error::RequestFailed)?;

        let len = read_content_length(&response).ok_or(Error::EmptyBody)?;

        let mut pb = ProgressBar::new(len);
        pb.set_units(Units::Bytes);

        let mut reader = BufReader::new(response.into_reader());

        let len = copy(&mut reader, &mut writer, |len| {
            pb.add(len);
        });

        pb.finish();

        len
    }
}

impl Default for Downloader {
    fn default() -> Downloader {
        Downloader {
            client: ureq::agent(),
        }
    }
}

fn read_content_length(response: &ureq::Response) -> Option<u64> {
    response
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
}

fn copy<R, W, F>(reader: &mut R, writer: &mut W, mut cb: F) -> Result<u64, Error>
where
    R: Read,
    W: Write,
    F: FnMut(u64),
{
    let mut buf = [0; DEFAULT_BUF_SIZE];
    let mut written = 0;

    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(len) => len,
            Err(e) => return Err(Error::Io(e)),
        };

        writer.write_all(&buf[..len]).map_err(Error::Io)?;
        written += len as u64;
        cb(len as u64);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy() {
        let mut src: &[u8] = b"hello";
        let mut dst: Vec<u8> = Vec::new();

        let mut written = 0;

        let len = copy(&mut src, &mut dst, |len| written += len).unwrap();

        assert_eq!(&b"hello"[..], &dst[..]);
        assert_eq!(written, len);
    }
}
