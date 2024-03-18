use isahc::{http::{HeaderMap, HeaderName, HeaderValue}, Body, Response};
use log::{error, warn, info, debug, trace};
use toml::{map::Map, Value::{self, Table}};
use serde::Deserialize;
use std::process;

#[derive(Deserialize)]
struct HeaderRuleDefinition {
    multiplier: u8,
    negative: Vec<String>,
    positive: Vec<String>,
    required: bool,
}

pub fn run(response: &Response<Body>, scan_type: String, rules: Value) -> (u32, u32) {
    let mut passed_tests: u32 = 0;
    let mut total_tests: u32 = 0;

    match scan_type.as_str() {
        "headers" => {
            (passed_tests, total_tests) = test_headers(&rules, response.headers());
        }
        _ => {
            warn!("Don't know how to handle this type of test ({}) yet, skipping", scan_type);
        }
    };
    let percentage = (passed_tests as f64 / total_tests as f64) * 100.0;
    info!("Score: {}/{} ({:.1}%) {}", passed_tests, total_tests, percentage, generate_grade(percentage));
    (passed_tests, total_tests)
}

fn test_headers(rules: &Value, headers: &HeaderMap) -> (u32, u32) {
    let mut added_passed: u32 = 0;
    let mut added_total: u32 = 0;

    let mut untested_headers: Vec<String> = Vec::new();

    // iterate over presented headers
    if let Table(table) = rules {
        // get all the require defined headers
        for (header_rule, definition) in table {
            if header_rule == "metadata" { continue; }
            let header_definition: HeaderRuleDefinition = to_header_rule_definition(definition, header_rule);
            if header_definition.required {
                untested_headers.push(header_rule.to_string());
            }
        }
        // process the headers present in the response
        for header in headers {
            let (new_passed, new_total, tested_header) = handle_headers_definition_table(header, table);
            for (i, header) in untested_headers.iter().enumerate() {
                if header == &tested_header {
                    untested_headers.remove(i);
                    break;
                }
            }
            added_passed += new_passed;
            added_total += new_total;
        }
        // process untested headers
        for header in untested_headers.iter() {
            let (new_passed, new_total) = handle_untested_required_headers(header.to_string(), table);
            added_passed += new_passed;
            added_total += new_total;
        }
        debug!("Required untested headers: {:?}", untested_headers);
    }
    (added_passed, added_total)
}

fn to_header_rule_definition(toml_value: &Value, header_name: &String) -> HeaderRuleDefinition {
    // FIXME: There has to be a better way of doing this...
    let toml_definition_sanitize = toml_value.to_string()
        .replace("\",", "\".")  // Preserve arrays for what's to come
        .replace(", ", "\n")    // Convert serialize comma separators to TOML newline
        .replace("{ ", "")      // Delete starting bracket
        .replace(" }", "")      // Delete ending bracket
        .replace("\".", "\","); // Covert preserved arrays back into arrays
    let header_definition: Result<HeaderRuleDefinition, toml::de::Error> = toml::from_str(&toml_definition_sanitize);
    match header_definition {
        Ok(definition) => definition,
        Err(e) => {
            error!("Problem parsing rule file");
            error!("[{}] is {}", header_name, e.message());
            process::exit(4);
        }
    }
}

fn handle_headers_definition_table(header: (&HeaderName, &HeaderValue), table: &Map<String, Value>) -> (u32, u32, String) {
    let mut added_passed: u32 = 0;
    let mut added_total: u32 = 0;
    let mut header_tested = String::from("");

    for (header_rule, definition) in table {
        if header_rule == "metadata" { continue; }

        let header_definition: HeaderRuleDefinition = to_header_rule_definition(definition, header_rule);

        if check_header_is_present(header, header_rule.to_string()) {
            let (new_passed, new_total) = process_header_rule_definition(header, &header_definition);
            added_passed += new_passed;
            added_total += new_total;
            header_tested.push_str(header_rule.as_str());
            break;
        }
    }
    (added_passed, added_total, header_tested)
}

fn check_header_is_present(header: (&HeaderName, &HeaderValue), current_rule: String) -> bool {
    if current_rule == header.0.to_string() {
        debug!("► {}", current_rule);
        return true;
    }
    trace!("Group ({}) != ({:?})", current_rule, header.0);
    false
}

fn process_header_rule_definition(header: (&HeaderName, &HeaderValue), definition: &HeaderRuleDefinition) -> (u32, u32) {
    let mut passed_tests: u32 = 0;
    let mut total_tests: u32 = 0;
    let header_value_string = header.1.to_str().unwrap().to_string();

    let indent = 2;

    debug!("{}▼ Negative", " ".repeat(indent));
    for neg in &definition.negative {
        debug!("{}▼ {}", " ".repeat(indent * 2), neg);
        let mut detected = false;
        let is_required = !neg.contains('?');
        for rule in split_rule(neg) {
            if neg == "present" {
                debug!("{}❌ Header was present", " ".repeat(indent * 3));
                detected = true;
                break;
            }
            if header_value_string.contains(&rule) {
                debug!("{}❌ {} was present", " ".repeat(indent * 3), rule);
                detected = true;
                break;
            } else {
                debug!("{}✅ {} was not present", " ".repeat(indent * 3), rule);
            }
        }
        if is_required && detected  {
            total_tests += 1;
        } else if is_required && !detected {
            passed_tests += 1;
            total_tests += 1;
        }
    }

    debug!("{}▼ Positive", " ".repeat(indent));
    for pos in &definition.positive {
        debug!("{}▼ {}", " ".repeat(indent * 2), pos);
        let mut detected = false;
        let is_required = !pos.contains('?');
        trace!("Test start");
        for rule in split_rule(pos) {
            if pos == "present" {
                debug!("{}✅ Header was present", " ".repeat(indent * 3));
                detected = true;
                break;
            }
            if header_value_string.contains(&rule) {
                debug!("{}✅ {} was present", " ".repeat(indent * 3), rule);
                detected = true;
                break;
            } else {
                debug!("{}❌ {} was not present", " ".repeat(indent * 3), rule);
            }
        }
        if is_required && detected  {
            total_tests += 1;
            passed_tests += 1;
        } else if !is_required && detected {
            passed_tests += 1;
        } else if is_required && !detected {
            total_tests += 1;
        }
    }
    (passed_tests * definition.multiplier as u32, total_tests * definition.multiplier as u32)
}

fn handle_untested_required_headers(untested_header: String, table: &Map<String, Value>) -> (u32, u32) {
    let mut added_total: u32 = 0;
    let mut added_passed: u32 = 0;

    for (header_rule, definition) in table {
        if header_rule == "metadata" { continue; }
        if header_rule == &untested_header {
            let header_definition: HeaderRuleDefinition = to_header_rule_definition(definition, header_rule);
            if header_definition.negative.iter().any(|e| e == "present") {
                let length = header_definition.negative.len() as u32;
                added_total += length * header_definition.multiplier as u32;
                added_passed += length * header_definition.multiplier as u32;
            }
            added_total += header_definition.positive.len() as u32* header_definition.multiplier as u32;
        }
    }
    (added_passed, added_total)
}

fn split_rule(rule: &str) -> Vec<String> {
    let fragments = rule.split('|');
    let mut fragment_vec = Vec::new();

    for fragment in fragments {
        fragment_vec.push(fragment.to_string().replace('?', ""));
    }

    fragment_vec
}

fn generate_grade (percentage: f64) -> String {
    if percentage > 100.0 {
        String::from("A+")
    } else if percentage > 80.0 {
        String::from("A")
    } else if percentage > 70.0 {
        String::from("B")
    } else if percentage > 60.0 {
        String::from("C")
    } else if percentage > 50.0 {
        String::from("D")
    } else if percentage > 40.0 {
        String::from("E")
    } else if percentage > 30.0 {
        String::from("F")
    } else {
        String::from("U")
    }
}
