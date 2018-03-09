use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

pub struct Version {
    pub name: String,
    pub pathname: PathBuf,
    pub filesize: u64,
}

impl Version {
    pub fn new<P>(name: &str, pathname: P) -> io::Result<Version> where P: AsRef<Path> {
        Ok(Version {
            name: name.to_owned(),
            pathname: pathname.as_ref().to_path_buf(),
            filesize: filesize(pathname)?,
        })
    }
}

pub fn optimize<P>(src: P) -> io::Result<Vec<Version>> where P: AsRef<Path> {
    Ok(vec![
        noop(&src)?,
        reencode(&src)?,
        recompress(&src)?,
    ])
}

fn noop<P>(src: P) -> io::Result<Version> where P: AsRef<Path> {
    Version::new("original", &src)
}

fn reencode<P>(src: P) -> io::Result<Version> where P: AsRef<Path> {
    let dst = tmp_pathname();
    let result = cjpeg(&src, &dst)?;

    if result.status.success() {
        Version::new("reencoded", &dst)
    } else {
        let message = String::from_utf8_lossy(&result.stderr);
        Err(io::Error::new(io::ErrorKind::Other, message))
    }
}

fn recompress<P>(src: P) -> io::Result<Version> where P: AsRef<Path> {
    let dst = tmp_pathname();
    let result = jpegtran(&src, &dst)?;

    if result.status.success() {
        Version::new("recompressed", &dst)
    } else {
        let message = String::from_utf8_lossy(&result.stderr);
        Err(io::Error::new(io::ErrorKind::Other, message))
    }
}

fn filesize<P>(path: P) -> io::Result<u64> where P: AsRef<Path> {
    fs::metadata(path).map(|m| m.len())
}

fn tmp_pathname() -> PathBuf {
    let id = Uuid::new_v4();
    let filename = format!("{}.jpg", id.hyphenated());

    let mut pathname = env::temp_dir();
    pathname.push(&filename);

    pathname
}

fn search_path() -> String {
    env::var("MOZJPEG_HOME").unwrap_or(String::from("/usr/local/opt/mozjpeg"))
}

fn cjpeg_bin() -> PathBuf {
    [&search_path(), "bin", "cjpeg"].iter().collect()
}

fn cjpeg<P, Q>(src: P, dst: Q) -> io::Result<Output>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    Command::new(cjpeg_bin())
        .arg("-quality").arg("90")
        .arg("-outfile").arg(dst.as_ref())
        .arg(src.as_ref())
        .output()
}

fn jpegtran_bin() -> PathBuf {
    [&search_path(), "bin", "jpegtran"].iter().collect()
}

fn jpegtran<P, Q>(src: P, dst: Q) -> io::Result<Output>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    Command::new(jpegtran_bin())
        .arg("-copy").arg("none")
        .arg("-outfile").arg(dst.as_ref())
        .arg(src.as_ref())
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;

    static FIXTURE_PATHNAME: &'static str = "test/fixtures/96x96-q100.jpg";

    #[test]
    fn test_filesize() {
        assert_eq!(filesize(FIXTURE_PATHNAME).unwrap(), 828);
    }

    #[test]
    fn test_optimize() {
        let versions = optimize(FIXTURE_PATHNAME).unwrap();
        assert_eq!(versions.len(), 3);
    }
}
