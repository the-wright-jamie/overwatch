use log::{info, debug};

pub fn validate(target: &str) -> Result<(), isahc::Error> {
    info!("Validating target...");

    isahc::get(target)?;

    info!("OK!");
    debug!("Was able to validate the target");

    Ok(())
}
