mod channel;
mod exclusions;
mod package;
mod parse;
mod version;

use crate::exclusions::print_inclusions;
use channel::Channel;
use clap::Parser;
use color_eyre::Result;
use exclusions::PrintType;

#[derive(Parser)]
#[command(author, version, about)]
pub(crate) struct Cli {
    /// search term to filter packages with
    #[arg(required = true)]
    pub(crate) search_term: String,
    /// the release channel to find packages from
    #[arg(value_enum, long, short, default_value_t = Channel::Alpha)]
    pub(crate) channel: Channel,
    /// search term to filter packages with
    #[arg(required = false, long, default_value_t = false)]
    pub(crate) print_includes: bool,
    /// stuff
    #[arg(value_enum, long, default_value_t = PrintType::IncludeGroup)]
    pub(crate) print_type: PrintType,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    if cli.print_includes {
        print_inclusions(cli.print_type).await;
        return Ok(());
    }
    let packages = parse::parse(&cli.search_term, cli.channel).await?;
    if packages.is_empty() {
        println!("No results found!");
    } else {
        for package in &packages {
            println!("{package}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::Cli;
    use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
    use std::process::Command;

    #[test]
    fn cli_assert() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }

    #[test]
    fn cli_help() {
        assert_cmd_snapshot!(Command::new(get_cargo_bin("adx")).arg("--help"));
    }

    #[test]
    fn cli_search() {
        assert_cmd_snapshot!(Command::new(get_cargo_bin("adx")).arg("appcompat"));
    }

    #[test]
    fn cli_search_stable() {
        assert_cmd_snapshot!(
            Command::new(get_cargo_bin("adx"))
                .arg("--channel")
                .arg("stable")
                .arg("appcompat")
        );
    }

    #[test]
    fn cli_search_no_results() {
        assert_cmd_snapshot!(Command::new(get_cargo_bin("adx")).arg("qtc"));
    }
}
