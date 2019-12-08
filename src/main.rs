extern crate clap;
extern crate log;
extern crate reqwest;
extern crate roxmltree;

use clap::{App, Arg};
use log::{Level, LevelFilter, Metadata, Record};
mod parse;

/// Simple logger that simply outputs everything using println!()
/// It prints all levels in debug builds, and Level::Info and below
/// on release builds.
struct StdOutLogger;

impl log::Log for StdOutLogger {
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

static LOGGER: StdOutLogger = StdOutLogger;

fn main() {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
    let matches = App::new("androidx-release-watcher")
        .version("0.1")
        .author("Harsh Shandilya <msfjarvis@gmail.com>")
        .about("Notify about the latest happenings in the Google Maven world")
        .args(&[
            Arg::with_name("package")
                .short("p")
                .long("package")
                .takes_value(true)
                .help("Name of package to filter in the results"),
            Arg::with_name("detailed")
                .short("d")
                .long("detail")
                .help("Output detailed information on each package"),
        ])
        .get_matches();
    match crate::parse::parse(matches.value_of("package").unwrap_or("").to_string()) {
        Ok(packages) => {
            if matches.is_present("detailed") {
                for package in packages.iter() {
                    println!("{}", package);
                }
            } else {
                for package in packages.iter() {
                    println!("{:?}", package);
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}
