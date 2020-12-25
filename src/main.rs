mod package;
mod parse;
mod stability;

use clap::{crate_authors, crate_description, crate_name, crate_version, AppSettings, Clap};

#[derive(Clap)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    setting = AppSettings::ColoredHelp,
    setting = AppSettings::DeriveDisplayOrder,
)]
pub(crate) struct Cli {
    #[clap(default_value = "")]
    pub(crate) search_term: String,
}

fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    match crate::parse::parse(&cli.search_term) {
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
