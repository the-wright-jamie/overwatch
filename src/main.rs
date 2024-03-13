// imports
use clap::Parser;
use simplelog::*;
use std::process;
use log::error;

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
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Debug mode: show debug messages
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,
}

fn main() {
    // Get the passed in arguments
    let arguments = Parameters::parse();

    // Set logging levels
    let logging_init = TermLogger::init(
        if      arguments.quiet   { LevelFilter::Warn  }
        else if arguments.verbose { LevelFilter::Debug }
        else                      { LevelFilter::Info  },
        Config::default(), TerminalMode::Mixed, ColorChoice::Auto
    );

    if logging_init.is_err() {
        println!("Something went wrong when initating the logger. As a result, this program cannot continue.");
        process::exit(1);
    }

    drop(logging_init);

    // Change it so that we can dynamically add modules
    let target_check = modules::validate_target::validate(&arguments.target);
    if let Err(err) = target_check {
        error!("{}", err);
        process::exit(2);
    }
    drop(target_check);

    modules::check_headers::run(&arguments.target);
}
