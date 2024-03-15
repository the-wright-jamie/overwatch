use isahc::{Body, Response};
use log::{info, debug};

pub fn run(target: &str) -> Result<Response<Body>, isahc::Error> {
    info!("Accessing {}...", target);

    let response = isahc::get(target);

    match response {
        Ok(res) => {
            debug!("{}", res.status());
            Ok(res)
        },
        Err(err) => {
            Err(err)
        },
    }
}
