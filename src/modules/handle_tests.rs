use isahc::{http::{HeaderMap, HeaderName, HeaderValue}, Body, Response};
use log::{info, debug, trace, warn};
use toml::{map::Map, Value::{self, Table}};
use serde::Deserialize;

#[derive(Deserialize)]
struct HeaderRuleDefinition {
    multiplier: u8,
    negative: Vec<String>,
    positive: Vec<String>,
    required: bool,
}

// pub: allow access from outside
pub fn run(response: &Response<Body>, scan_type: String, rules: Value) -> (u32, u32) {
    let mut passed_tests: u32 = 0;
    let mut total_tests: u32 = 0;

    match scan_type.as_str() {
        "headers" => {
            let (added_passed, added_total) = handle_headers_test(&rules, response.headers());
            passed_tests += added_passed;
            total_tests += added_total;
        }
        _ => {
            warn!("Don't know how to handle this type of test ({}) yet, skipping", scan_type);
        }
    };
    let percentage = (passed_tests as f64 / total_tests as f64) * 100.0;
    info!("Score: {}/{} ({}%)", passed_tests, total_tests, percentage);
    (passed_tests, total_tests)
}

fn handle_headers_test(rules: &Value, headers: &HeaderMap) -> (u32, u32) {
    let mut added_passed: u32 = 0;
    let mut added_total: u32 = 0;

    // iterate over presented headers
    if let Table(table) = rules {
        for header in headers {
            let (new_passed, new_total) = handle_headers_table(header, table);
            added_passed += new_passed;
            added_total += new_total;
        }
    }
    (added_passed, added_total)
}

fn handle_headers_table(header: (&HeaderName, &HeaderValue), table: &Map<String, Value>) -> (u32, u32) {
    let mut added_passed: u32 = 0;
    let mut added_total: u32 = 0;

    for (header_rule, definition) in table {
        if header_rule == "metadata" {
            continue;
        }

        // FIXME: There has to be a better way of doing this...
        let toml_definition_sanitize = definition.to_string()
            .replace("\",", "\".")
            .replace(", ", "\n")
            .replace("{ ", "")
            .replace(" }", "")
            .replace("\".", "\",");
        let header_definition: HeaderRuleDefinition = toml::from_str(&toml_definition_sanitize).unwrap();

        if check_header_is_present(header, header_rule.to_string()) {
            let (new_passed, new_total) = process_header_rule_definition(header, &header_definition);
            added_passed += new_passed;
            added_total += new_total;
            break;
        }
    }
    (added_passed, added_total)
}

fn check_header_is_present(header: (&HeaderName, &HeaderValue), current_rule: String) -> bool {
    if current_rule == header.0.to_string() {
        debug!("Rule ({}) = ({:?})", current_rule, header.0);
        return true;
    }
    trace!("Rule ({}) != ({:?})", current_rule, header.0);
    false
}

fn process_header_rule_definition(header: (&HeaderName, &HeaderValue), definition: &HeaderRuleDefinition) -> (u32, u32) {
    let mut passed_tests: u32 = 0;
    let mut total_tests: u32 = 0;
    for neg in &definition.negative {
        let header_value_string = header.1.to_str().unwrap().to_string();
        if header_value_string.contains(neg) {
            total_tests += 1;
        } else {
            passed_tests += 1;
            total_tests += 1;
        }
    }
    for pos in &definition.positive {
        let header_value_string = header.1.to_str().unwrap().to_string();
        if header_value_string.contains(pos) {
            passed_tests += 1;
            total_tests += 1;
        } else {
            total_tests += 1;
        }

    }
    (passed_tests * definition.multiplier as u32, total_tests * definition.multiplier as u32)
}

// fn discover_missed_header_points(discovered_headers: Vec<String>, all_scannable_headers: Vec<String>) -> u32 {
//     let mut missed_headers
// }
