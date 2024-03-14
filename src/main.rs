// imports
use clap::Parser;
use simplelog::*;
use std::{process, path::Path, fs};
use log::{error, info, debug};
use crate::modules::{handle_rule_file, handle_tests, save_response};

// modules
mod modules;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Parameters {
    /// What website/domain to scan
    #[arg()]
    target: String,

    /// Write a report
    #[arg(short, long)]
    report: bool,

    /// Quiet mode: only show errors and warnings
    #[arg(short, long, conflicts_with = "verbose", conflicts_with = "loud")]
    quiet: bool,

    /// Debug mode: show debug messages
    #[arg(short, long, conflicts_with = "quiet", conflicts_with = "loud")]
    verbose: bool,

    /// Trace mode: show all messages
    #[arg(long, conflicts_with = "quiet", conflicts_with = "verbose")]
    loud: bool,
}

fn main() {
    // Get the passed in arguments
    let arguments = Parameters::parse();

    // Set logging levels
    let logging_init = TermLogger::init(
        if      arguments.quiet   { LevelFilter::Warn  }
        else if arguments.verbose { LevelFilter::Debug }
        else if arguments.loud    { LevelFilter::Trace }
        else                      { LevelFilter::Info  },
        Config::default(), TerminalMode::Mixed, ColorChoice::Auto
    );

    if logging_init.is_err() {
        println!("Something went wrong when initating the logger. As a result, this program cannot continue.");
        process::exit(1);
    }

    info!("---------- OVERWATCH ---------");

    let rules_dir_str = "rules";

    if !Path::new(rules_dir_str).exists() {
        // TODO: Download rules from GitHub
        error!("Rules directory doesn't exist. Without this, this program has no way of knowing what to scan and how to handle it");
        process::exit(2);
    }

    let get_response = save_response::run(&arguments.target);

    let response = match get_response {
        Ok(response) => response,
        Err(err) => {
            error!("Could not get headers from target. Please check spelling and internet connection.");
            error!("{}", err);
            process::exit(3);
        },
    };

    info!("Initialisation complete");

    // Already checked that the dir exists, so there shouldn't be a problem reading it
    // ...hopefully we aren't in a filesystem that's write only!
    // Allows a user to define custom rules for tests, and allows us to be
    // as generic as possible
    let rules_dir = fs::read_dir(rules_dir_str).unwrap();
    for entry in rules_dir {
        match entry {
            Ok(entry) => {
                debug!("Handling rule file {:?}", entry.path());
                let (scan_type, rules) = handle_rule_file::run(entry.path().to_str().unwrap());
                info!("Running tests from {:?}", entry.path());
                handle_tests::run(&response, scan_type, rules);
            },
            Err(err) => eprintln!("Error reading entry: {}", err),
        }
    }
}
