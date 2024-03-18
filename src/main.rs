use clap::Parser;
use simplelog::*;
use std::{process, path::Path, fs};
use log::{error, info, debug};
use crate::modules::{handle_rule_file, handle_tests, save_response};

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

    /// Allows the user to define a custom rules folder location
    #[arg(long, default_value_t = String::from("rules"))]
    custom_rules_directory: String,

    /// Deactivates automatic redirects
    #[arg(short, long)]
    no_redirect: bool,
}

fn main() {
    let arguments = Parameters::parse();

    let logging_init = TermLogger::init(
        if      arguments.quiet   { LevelFilter::Warn  }
        else if arguments.verbose { LevelFilter::Debug }
        else if arguments.loud    { LevelFilter::Trace }
        else                      { LevelFilter::Info  },
        Config::default(), TerminalMode::Mixed, ColorChoice::Auto
    );

    if logging_init.is_err() {
        println!("Something went wrong when initiating the logger. As a result, this program cannot continue.");
        process::exit(1);
    }

    info!("---------- OVERWATCH ----------");

    let rules_dir_str = &arguments.custom_rules_directory;

    if !Path::new(rules_dir_str).exists() {
        // TODO: Download rules from GitHub
        error!("Rules directory doesn't exist. Without this, this program has no way of knowing what to scan and how to handle it");
        process::exit(2);
    }

    let get_response = save_response::run(&arguments.target, &arguments.no_redirect);

    let response = match get_response {
        Ok(response) => response,
        Err(err) => {
            error!("{}", err);
            process::exit(3);
        },
    };

    // Already checked that the dir exists before this point,
    // so there shouldn't be a problem reading it...hopefully we aren't
    // in a filesystem that's write only!

    // Reading in TOML files that define tests allows
    //     1. a user to define custom rules for tests
    //     2. allows us to be as generic as possible

    // The following is a very generic way of doing this, which is good
    // as it lets us keep as much logic as possible out of the main function
    let rules_dir = fs::read_dir(rules_dir_str).unwrap();

    // Loop over all the rule files in the rules directory.
    for entry in rules_dir {
        match entry {
            Ok(entry) => {
                debug!("Processing rule file {:?}", entry.path());
                if entry.path().to_str().unwrap().contains("example") || entry.path().to_str().unwrap().contains("skip") {
                    debug!("File had \"example\" or \"skip\" in its name, skipping");
                    continue;
                }
                let (title, scan_type, rules) = handle_rule_file::run(entry.path().to_str().unwrap());
                info!("Running \"{}\" test from {:?}", title, entry.path());
                handle_tests::run(&response, scan_type, rules);
            },
            Err(err) => eprintln!("Error reading entry: {}", err),
        }
    }
}
