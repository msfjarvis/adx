extern crate clap;
extern crate log;
extern crate reqwest;
extern crate roxmltree;

use clap::{App, Arg};
use log::{Level, Metadata, Record};
mod parse;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if cfg!(debug_assertions) {
            true
        } else {
            metadata.level() <= Level::Info
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() {
    let matches = App::new("androidx-release-watcher")
        .version("0.1")
        .author("Harsh Shandilya <msfjarvis@gmail.com>")
        .about("Notify about the latest happenings in the Google Maven world")
        .arg(
            Arg::with_name("package")
            .short("p")
            .long("package")
            .takes_value(true)
            .help("Name of package to filter in the results")
        )
        .get_matches();
    match crate::parse::parse_packages(matches.value_of("package").unwrap_or("")) {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}
