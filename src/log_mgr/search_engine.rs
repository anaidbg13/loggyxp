
pub fn search_word(content: String, word: String) -> bool
{
    let mut found = false;
    if content.contains(&word){
        found = true;
    }
   return found;
}

pub fn pattern_frequency(content: String, pattern: String) -> usize
{
    return content.match_indices(&pattern).map(|(i, _)| i as i32).collect::<Vec<i32>>().len();

}