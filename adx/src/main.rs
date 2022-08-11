mod channel;
mod package;
mod parse;
#[cfg(feature = "measure-alloc")]
mod stats_alloc;

#[cfg(feature = "measure-alloc")]
use std::alloc::System;

use channel::Channel;
use clap::{AppSettings, Parser};
use color_eyre::Result;
#[cfg(feature = "measure-alloc")]
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

#[cfg(feature = "measure-alloc")]
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub(crate) struct Cli {
    /// search term to filter packages with
    #[cfg(not(feature = "measure-alloc"))]
    #[clap(required = true)]
    pub(crate) search_term: String,
    /// the release channel to find packages from
    #[clap(short = 'c', long = "channel", value_parser, default_value = "alpha")]
    pub(crate) channel: Channel,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    #[cfg(feature = "measure-alloc")]
    let reg = Region::new(GLOBAL);
    let cli = Cli::parse();
    #[cfg(feature = "measure-alloc")]
    let packages = parse::parse("", cli.channel).await?;
    #[cfg(not(feature = "measure-alloc"))]
    let packages = parse::parse(&cli.search_term, cli.channel).await?;
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

#[cfg(test)]
mod test {
    use super::Cli;

    #[test]
    fn cli_assert() {
        <Cli as clap::CommandFactory>::command().debug_assert()
    }
}
