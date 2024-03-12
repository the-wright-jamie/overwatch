use curl::easy::Easy;
use log::{info, debug};

// Result is used for error checking
pub fn validate(target: &str) -> Result<u32, String> {
    info!("Validating target...");
    debug!("Using cURL to validate the target");

    let mut easy = Easy::new();
    easy.url(target).unwrap();
    let target_validation = easy.perform();
    if target_validation.is_err() {
        return Err("Could not reach target. Please check your internet connection, and the spelling of the target.".to_string());
    }

    debug!("Was able to validate the target");

    Ok(0)
}
