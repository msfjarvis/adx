extern crate clap;
extern crate indicatif;
extern crate log;
extern crate roxmltree;
extern crate ureq;

use clap::{App, Arg, ArgGroup};
use log::{LevelFilter, Metadata, Record};

mod channel;
mod parse;

/// Simple logger that simply outputs everything using println!()
/// It prints all levels in debug builds, and nothing on release builds.
struct StdOutLogger;

impl log::Log for StdOutLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        cfg!(debug_assertions)
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
        .about("Poll Google's Maven repository to fetch the latest versions of AndroidX packages")
        .args(&[
            Arg::with_name("package")
                .help("Name of package to filter in the results")
                .index(1),
            Arg::with_name("all")
                .short("a")
                .long("all")
                .takes_value(false),
            Arg::with_name("condensed")
                .short("c")
                .long("condensed")
                .help("Only print the latest version of the package"),
        ])
        .group(
            ArgGroup::with_name("search_term")
                .required(true)
                .args(&["package", "all"]),
        )
        .get_matches();
    match crate::parse::parse(matches.value_of("package").unwrap_or("")) {
        Ok(packages) => {
            if packages.is_empty() {
                println!("No results found!");
            } else if matches.is_present("condensed") {
                for package in packages.iter() {
                    println!("{:?}", package);
                }
            } else {
                for package in packages.iter() {
                    println!("{}", package);
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}
