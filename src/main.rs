#[macro_use] extern crate clap;
extern crate multimeta;
extern crate url;

use clap::{App, Arg};
use url::Url;
use multimeta::{Extractor, MelonExtractor};
use multimeta::renderer::Renderer;
use multimeta::writer::Writer;

fn main() {
    let matches = App::new("multimeta")
        .version(crate_version!())
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("DIR")
             .help("Set output directory")
             .default_value("."))
        .arg(Arg::with_name("artist-id")
             .help("The local artist ID")
             .index(1)
             .required(true))
        .arg(Arg::with_name("url")
             .help("The remote URL to scrape")
             .index(2)
             .required(true))
        .get_matches();

    let output_dir = matches.value_of("output").unwrap();
    let artist_id = matches.value_of("artist-id").unwrap();

    let raw_url = matches.value_of("url").unwrap();
    let url = Url::parse(raw_url).expect("malformed URL");

    let extractor = if MelonExtractor::matches(&url) {
        MelonExtractor::new(&url).unwrap()
    } else {
        panic!("failed to match a registered extractor");
    };

    let album = extractor.extract().expect("extraction failed");

    let renderer = Renderer::new();

    let writer = Writer::new(&output_dir);
    writer.write(&renderer, &artist_id, &album).expect("write failed");
}
