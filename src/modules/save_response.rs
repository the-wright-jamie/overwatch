use isahc::{Body, Response};
use log::{info, debug};

pub fn run(target: &str) -> Result<Response<Body>, isahc::Error> {
    info!("Getting response from target...");

    let response = isahc::get(target)?;

    info!("OK!");
    debug!("Was able to reach the target");

    Ok(response)
}
