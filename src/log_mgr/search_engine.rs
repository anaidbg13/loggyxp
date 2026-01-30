extern crate regex;
use regex::Regex;


pub fn search_string(content: &String, word: &String) -> Vec<String> {
    let mut lines = Vec::new();

    if word.is_empty() {
        return lines;
    }

    let needle = word.to_lowercase();

    for (idx, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(&needle) {
            lines.push(format!("{}: {}", idx + 1, line));
        }
    }

    lines
}







pub fn search_input_pattern(content: &String, pattern: &String) -> Vec<String>
{

    let mut matches: Vec<String> = Vec::new();

    match Regex::new(pattern) {
        Ok(regex) => {
            matches = content
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| {
                    regex.is_match(line)
                        .then(|| format!("{}: {}", idx + 1, line))
                })
                .collect();
            return matches;
        }
        Err(_) => {
            println!("Invalid pattern");
            matches.push(String::from("loggyxp: invalid regex pattern"));
            return matches;
        }
    }
}
