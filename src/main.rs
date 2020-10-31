#[macro_use]
extern crate clap;
use clap::ArgMatches;

mod cli;
mod hooks;
mod providers;
use cli::build_cli;
mod config;
use config::Config;

fn main() {
    let matches = build_cli().get_matches();

    // Handle CLI subcommands
    match matches.subcommand() {
        ("check", Some(matches)) => check_for_updates(matches),
        ("query", Some(matches)) => query_data(matches),
        _ => std::process::exit(1),
    }
}

/// Check upstream provider for updates
/// If there are updates run all associated hooks, else just end
fn check_for_updates(matches: &ArgMatches) {
    let file = matches.value_of("FILE").unwrap();
    let config = Config::from_file(file);

    if let Some(data) = config.provider.poll() {
        // We have data, let's run each of the hooks in order
        // If there is no data, just exit the program with nothing more to do.
        for hook in config.hooks {
            match hook.run(&data) {
                Err(e) => {
                    eprintln!("Error running hook {:?}", e);
                    std::process::exit(exitcode::SOFTWARE);
                }
                Ok(()) => {}
            }
        }
    }
}

/// Check local cache and print out the latest
/// version of the data we have
fn query_data(matches: &ArgMatches) {
    let file = matches.value_of("FILE").unwrap();
    let config = Config::from_file(file);

    match config.provider.query() {
        Ok(data) => println!("{}", data),
        Err(e) => {
            eprintln!("Error fetching data from cache: {:?}", e);
            std::process::exit(exitcode::SOFTWARE);
        }
    }
}
