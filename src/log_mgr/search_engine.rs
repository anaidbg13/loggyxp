extern crate regex;
use self::regex::Regex;

pub fn search_string(content: &String, word: &String) -> bool
{
  /*
    Keep  to understand references
    let mut found = false;
    if content.contains(&*word){
        found = true;
    }
   return found;*/

    let found;
    let pattern = format!(r"\b{}\b", word);
    let re = Regex::new(&pattern).unwrap();
    if re.is_match(content){
        println!("{} appears: {} times",word, re.find_iter(word).count());
        found = true;
        return found;
    }else
    {
        println!("{} was not found in log", word);
        found = false;
        return found;
    }


}

pub fn search_input_pattern(content: &String, pattern: &String) -> Vec<String>
{

    let mut matches: Vec<String> = Vec::new();

    match Regex::new(pattern) {
        Ok(regex) => {
            // Collect all matches as strings
                matches = regex
                .find_iter(content)
                .map(|m| m.as_str().to_string())
                .collect();

            if !matches.is_empty() {
                println!("{} has {} matches", pattern, matches.len());
                matches.push(String::from("valid"));
                return matches;
            } else {
                println!("Could not find any matches");
                matches.push(String::from("invalid"));
                return matches;
            }
        }
        Err(_) => {
            println!("Invalid pattern");
            matches.push(String::from("invalid"));
            return matches;
        }
    }

    /*if let Ok(re) = Regex::new(&pattern) {
        println!("Regex is valid!");
    } else {
        eprintln!("Regex pattern is invalid");
    }*/
}

pub fn remove_duplicates(matches: &Vec<String>) -> Vec<String>
{
    let mut non_duplicates = matches.clone();

    non_duplicates.sort();
    non_duplicates.dedup();

    return non_duplicates;

}