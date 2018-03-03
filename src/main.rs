#[macro_use] extern crate clap;
extern crate multimeta;
extern crate url;

use clap::{App, Arg};
use url::Url;
use multimeta::{editor, extractors};
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

    let album = extractors::factory(&url)
        .and_then(|e| e.extract())
        .map(|a| editor::edit(&a))
        .unwrap_or_else(|e| panic!("{:?}", e));

    let renderer = Renderer::new();
    let writer = Writer::new(&output_dir);
    writer.write(&renderer, &artist_id, &album).expect("write failed");
}
