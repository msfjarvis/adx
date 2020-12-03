mod parse;
mod stability;

use clap::{crate_name, crate_version, App, Arg, ArgGroup};

fn main() {
    pretty_env_logger::init();
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author("Harsh Shandilya <me@msfjarvis.dev>")
        .about("Poll Google's Maven repository to fetch the latest versions of AndroidX packages")
        .args(&[Arg::with_name("package")
            .help("Name of package to filter in the results")
            .index(1)])
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
            } else {
                for package in packages.iter() {
                    println!("{}", package);
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}
