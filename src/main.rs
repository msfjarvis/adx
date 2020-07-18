mod parse;

use clap::{crate_name, crate_version, App, Arg, ArgGroup};
use log::{LevelFilter, Metadata, Record};

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
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author("Harsh Shandilya <me@msfjarvis.dev>")
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
            } else if matches.is_present("condensed") || matches.is_present("all") {
                for package in packages.iter() {
                    println!("{}", package.get_condensed());
                }
            } else {
                println!("{}", packages[0]);
                for package in packages.iter().skip(1) {
                    println!("{}", package)
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}
