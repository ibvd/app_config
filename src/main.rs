#[macro_use]
extern crate clap;
use clap::ArgMatches;

use simple_eyre::eyre::{WrapErr, Report};

mod cli;
mod hooks;
mod providers;
use cli::build_cli;
mod config;
use config::Config;


fn main() -> Result<(), Report> {
    simple_eyre::install()?;

    run()?;

    Ok(())
}


fn run() -> eyre::Result<()> {
    let matches = build_cli().get_matches();

    // Handle CLI subcommands
    let res = match matches.subcommand() {
        ("check", Some(matches)) => check_for_updates(matches),
        ("query", Some(matches)) => query_data(matches),
        // ("params", Some(matches)) => params(matches),
        _ => std::process::exit(1),
    };

    res
}


/// Check upstream provider for updates
/// If there are updates run all associated hooks, else just end
fn check_for_updates(matches: &ArgMatches) -> eyre::Result<()> {
    let file = matches.value_of("FILE").unwrap();
    let config = Config::from_file(file);

    if let Some(data) = config.provider.poll()? {
        // We have data, let's run each of the hooks in order
        // If there is no data, just exit the program with nothing more to do.
        for hook in config.hooks {
            hook.run(&data).wrap_err("Error running hook")?;
        }
    }
    Ok(())
}


/// Check local cache and print out the latest
/// version of the data we have
fn query_data(matches: &ArgMatches) -> eyre::Result<()> {
    let file = matches.value_of("FILE").unwrap();
    let config = Config::from_file(file);

    let data = config.provider.query()?;
    println!("{}", data);
    Ok(())
}
