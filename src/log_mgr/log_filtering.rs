extern crate serde_json;

use json::value;
use self::serde_json::Value;
pub fn filter_lines(content: &String, pattern: &String) -> String
{
    let mut new_lines: String = String::new();
    for line in content.lines() {
        if line.contains(&*pattern) {
            new_lines += line;
            new_lines += "\n";
        }
    }
    return new_lines;
}

pub fn parse_json(content: &String) -> Result<(), Box<dyn std::error::Error>>
{

    for line in content.lines() {
        let line = line;
        let v: Value = serde_json::from_str(&line)?;
        println!("{}", serde_json::to_string_pretty(&v)?);
    }

    return Ok(());
}

pub fn filter_by_key_json(content: &String, key: &String) -> String
{
    let v: Value = serde_json::from_str(&content).unwrap();

    let results = filter_nested_keys(&v, key);

    results
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn filter_nested_keys<'a>(v: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut results = Vec::new();

    match v {
        Value::Object(map) => {
            if let Some(val) = map.get(key) {
                results.push(val);
            }
            for val in map.values() {
                results.extend(filter_nested_keys(val, key));
            }
        }
        Value::Array(arr) => {
            for val in arr {
                results.extend(filter_nested_keys(val, key));
            }
        }
        _ => {}
    }

    results
}
