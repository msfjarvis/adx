mod channel;
mod package;
mod parse;

use std::error::Error;

use channel::Channel;
use clap::{crate_authors, crate_description, crate_name, crate_version, AppSettings, Clap};
use std::result::Result;

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
    /// search term to filter packages with
    #[clap(required = true)]
    pub(crate) search_term: String,
    /// the release channel to find packages from
    #[clap(short='c', long="channel", possible_values=&["dev", "alpha", "beta", "rc", "stable"], default_value="alpha")]
    pub(crate) channel: Channel,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let packages = crate::parse::parse(&cli.search_term, cli.channel).await?;
    if packages.is_empty() {
        println!("No results found!");
    } else {
        for package in packages.iter() {
            println!("{}", package);
        }
    };
    Ok(())
}
