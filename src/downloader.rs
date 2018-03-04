use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use pbr::{ProgressBar, Units};
use reqwest::Client;
use reqwest::header::ContentLength;

const DEFAULT_BUF_SIZE: usize = 8192; // bytes

#[derive(Debug)]
pub enum Error {
    Io,
    RequestFailed,
    EmptyBody,
}

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader { client: Client::new() }
    }

    pub fn save<P>(&self, url: &str, dst: P) -> Result<u64, Error>
    where
        P: AsRef<Path>,
    {
        let file = File::create(dst).or(Err(Error::Io))?;
        let mut writer = BufWriter::new(file);

        let len = self.content_length(url)?;

        let mut pb = ProgressBar::new(len);
        pb.set_units(Units::Bytes);

        let res = self.client.get(url).send().or(Err(Error::RequestFailed))?;

        if !res.status().is_success() {
            return Err(Error::RequestFailed);
        }

        let mut reader = BufReader::new(res);

        let len = copy(&mut reader, &mut writer, |len| {
            pb.add(len);
        });

        pb.finish();

        len
    }

    fn content_length(&self, url: &str) -> Result<u64, Error> {
        let res = self.client.head(url).send().or(Err(Error::RequestFailed))?;

        if res.status().is_success() {
            res.headers().get::<ContentLength>()
                .map(|len| **len)
                .ok_or(Error::EmptyBody)
        } else {
            Err(Error::RequestFailed)
        }
    }
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
            Err(_) => return Err(Error::Io),
        };

        writer.write_all(&buf[..len]).or(Err(Error::Io))?;
        written += len as u64;
        cb(len as u64);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::copy;

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
