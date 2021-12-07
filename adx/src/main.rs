mod channel;
mod package;
mod parse;
#[cfg(feature = "measure-alloc")]
mod stats_alloc;

use channel::Channel;
use clap::{crate_authors, crate_description, crate_name, crate_version, AppSettings, Parser};
use color_eyre::Result;

#[cfg(feature = "measure-alloc")]
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
#[cfg(feature = "measure-alloc")]
use std::alloc::System;

#[cfg(feature = "measure-alloc")]
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[derive(Parser)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    setting = AppSettings::DeriveDisplayOrder,
)]
pub(crate) struct Cli {
    /// search term to filter packages with
    #[cfg(not(feature = "measure-alloc"))]
    #[clap(required = true)]
    pub(crate) search_term: String,
    /// the release channel to find packages from
    #[clap(short='c', long="channel", possible_values=&["alpha", "a", "beta", "b", "dev", "d", "rc", "r", "stable", "s"], default_value="a")]
    pub(crate) channel: Channel,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    #[cfg(feature = "measure-alloc")]
    let reg = Region::new(GLOBAL);
    let cli = Cli::parse();
    #[cfg(feature = "measure-alloc")]
    let packages = crate::parse::parse("", cli.channel).await?;
    #[cfg(not(feature = "measure-alloc"))]
    let packages = crate::parse::parse(&cli.search_term, cli.channel).await?;
    if packages.is_empty() {
        println!("No results found!");
    } else {
        for package in &packages {
            println!("{}", package);
        }
    };
    #[cfg(feature = "measure-alloc")]
    println!("{:#?}", reg.change());
    Ok(())
}
