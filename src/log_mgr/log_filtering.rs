

pub fn filter_lines(content: String, pattern: String) -> String
{
    let mut new_lines: String = String::new();
    for line in content.lines() {
        if line.contains(&pattern) {
            new_lines += line;
            new_lines += "\n";
        }
    }
    return new_lines;
}