#[macro_use] extern crate clap;
extern crate env_logger;
#[macro_use] extern crate log;
extern crate multimeta;
extern crate url;

use clap::{App, Arg};
use log::LevelFilter;
use url::Url;

use multimeta::{editor, extractors};
use multimeta::renderer::Renderer;
use multimeta::writer::Writer;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("DIR")
             .help("Set output directory")
             .default_value("."))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .arg(Arg::with_name("artist-id")
             .help("The local artist ID")
             .index(1)
             .required(true))
        .arg(Arg::with_name("url")
             .help("The remote URL to scrape")
             .index(2)
             .required(true))
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    }

    let output_dir = matches.value_of("output").unwrap();
    let artist_id = matches.value_of("artist-id").unwrap();

    let raw_url = matches.value_of("url").unwrap();
    let url = Url::parse(raw_url).expect("malformed URL");

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
    writer.write_templates(&renderer, &artist_id, &album).expect("write failed");
}
