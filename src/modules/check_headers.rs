// imports
use log::{error, info, debug};
use isahc::prelude::*;

// pub: allow access from outside
pub fn run(target: &str) -> Result<(), isahc::Error> {
    // Display some info to the user
    info!("Now checking headers...");
    debug!("Using cURL to get infomation about the target");

    let mut response = isahc::get(target)?;

    // Print some basic info about the response to standard output.
    let content_security_policy = response.headers().get("content-security-policy");
    match content_security_policy {
        None => error!("LMAO what happened"),
        Some(_) => info!("content-security-policy found"),
    }

    // println!("{:#?}", response.headers());

    // Read the response body as text into a string and print it.
    // print!("{}", response.text()?);

    Ok(())
}
