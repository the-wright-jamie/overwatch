use isahc::{http::StatusCode, Body, Response};
use log::{warn, info, debug};

pub fn run(target: &str, no_redirect: &bool) -> Result<Response<Body>, isahc::Error> {
    info!("Accessing {}...", target);

    let response = isahc::get(target);

    match response {
        Ok(mut res) => {
            debug!("{}", res.status());
            debug!("\n{:#?}", res.headers());
            while res.status() == StatusCode::MOVED_PERMANENTLY && !no_redirect {
                // Using these nasty unwraps here as we know the website exists - we got a redirect
                // request, so we can safely follow is. The problem is, if internet connection is
                // dropped somehow inbetween here then this will crash... Perhaps an edge
                // condition, but we'll see. If we get bug reports, we'll fix it.
                warn!("Website was redirected to {:?}", res.headers().get("Location").unwrap());
                debug!("Automatically following redirect. To deactivate this, pass -n (--no-redirect) to the command line.");
                let location = res.headers().get("Location").unwrap().to_str().unwrap();
                info!("Accessing {}...", location);
                res = isahc::get(location).unwrap();
                debug!("{}", res.status());
                debug!("{:#?}", res.headers());
            }
            if res.status() != StatusCode::OK {
                warn!("Non-200 response: {}", res.status());
            }
            Ok(res)
        },
        Err(err) => {
            Err(err)
        },
    }
}
