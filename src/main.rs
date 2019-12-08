extern crate clap;
extern crate reqwest;
extern crate roxmltree;

use clap::{App, Arg};
mod parse;

fn main() {
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
    match crate::parse::parse(
        matches.value_of("package").unwrap_or("").to_string(),
        matches.is_present("detailed"),
    ) {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}
