use std::collections::HashMap;
use try_catch::catch;
use curl::easy::Easy;
// use std::io::{stdout, Write};
use log::{error, warn, info, debug};

pub fn run(target: &str) {
    info!("Now checking headers...");
    debug!("Using cURL to get infomation about the target");

    let mut easy = Easy::new();
    let mut headers: HashMap<String, String> = HashMap::new();

    easy.url(target).unwrap();

    easy.header_function(move |buffer| {
        let line = std::str::from_utf8(buffer).unwrap();
        debug!("{}", line.replace('\n', ""));
        let mut parts = line.splitn(2, ':');
        let cloned_parts = parts.clone();
        if cloned_parts.count() == 2 {
            let key = parts.next().unwrap().trim().to_string();
            let value = parts.next().unwrap().trim().to_string();
            headers.insert(key, value);
        }
        true
    }).unwrap();

    easy.perform().unwrap();

    // for header in &headers {
    //
    // }

    debug!("Got {} from target", easy.response_code().unwrap());
}
