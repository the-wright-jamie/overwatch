use serde::Deserialize;
use toml::{map::Map, Value::{self, Table}};
use log::{debug, trace};
use std::fs;

#[derive(Deserialize)]
struct Metadata {
    title: String,
    scan_type: String,
}

pub fn run(entry: &str) -> (String, String, Value) {
    debug!("Loading rules from {:?}...", entry);
    let path = std::path::Path::new(entry);

    // Checks should be done before the following is being run - 
    // so we can assume it's save to unwrap with no error handling
    let file = fs::read_to_string(path).unwrap();
    let rules = file.parse::<Value>().unwrap();
    let (title, scan_type) = get_rules(&rules);
    (title, scan_type, rules)
}

fn get_rules(rules: &Value) -> (String, String) {
    if let Table(table) = rules {
        return handle_table(table);
    }
    (String::from(""), String::from(""))
}

fn handle_table(table: &Map<String, Value>) -> (String, String) {
    for (key, value) in table.iter() {
        // add more validation like a string can't exist in both positive and negative
        // and that other expected things are present
        if &key.to_string() == "metadata" {
            trace!("Metadata found in the rule file");
            let toml_metadata_sanitize = value.to_string().replace(", ", "\n").replace("{ ", "").replace(" }", "");
            let metadata: Metadata = toml::from_str(&toml_metadata_sanitize).unwrap();
            return (metadata.title, metadata.scan_type);
        }
    }
    panic!("Couldn't find scan type in the rule file!\nRun with the verbose flag (-v) to see which file this failed on.");
}

