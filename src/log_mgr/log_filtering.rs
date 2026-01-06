extern crate serde_json;
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
    Ok(())
}