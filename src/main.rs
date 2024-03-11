// modules
mod modules;

// use std::process;
use log::{error, info};
use clap::Parser;
use env_logger::Env;
use std::io::Write;
// use std::{error::Error, fs::File, io::Read};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Parameters {
    /// What website/domain to scan
    #[arg()]
    target: String,

    /// Check the headers of the target
    #[arg(short = 'H', long)]
    headers_check: bool,

    /// Write a report
    #[arg(short, long)]
    report: bool,
}

fn main() {
    let arguments = Parameters::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    modules::logo::display_greeting();
    if arguments.headers_check { modules::check_headers::run() }
    error!("{0}", arguments.target);
    info!("This should be seen");
}
