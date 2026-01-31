extern crate regex;
use regex::Regex;


// Searches for a substring (case-insensitive) in each line of the content.
// Returns matching lines with their line numbers.
pub fn search_string(content: &String, word: &String) -> Vec<String> {
    let mut lines = Vec::new();

    // If the search word is empty, return empty result.
    if word.is_empty() {
        return lines;
    }

    let needle = word.to_lowercase();

    // Iterate over each line and check if it contains the search word.
    for (idx, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(&needle) {
            lines.push(format!("{}: {}", idx + 1, line));
        }
    }

    lines
}

// Searches for lines matching a regex pattern in the content.
// Returns matching lines with their line numbers.
pub fn search_input_pattern(content: &String, pattern: &String) -> Vec<String>
{
    let mut matches: Vec<String> = Vec::new();

    // Try to compile the regex pattern.
    match Regex::new(pattern) {
        Ok(regex) => {
            // Filter lines that match the regex and collect them.
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
            // If the pattern is invalid, return an error message.
            println!("Invalid pattern");
            matches.push(String::from("loggyxp: invalid regex pattern"));
            return matches;
        }
    }
}
