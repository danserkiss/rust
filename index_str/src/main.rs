use std::collections::HashMap;

// replace with much longer text for testing
const LARGE_TEXT: &str = "Note that the above command required that you specify the file name explicitly . If you were to directly use rustc to compile a different program , a different command line invocation would be required . If you needed to specify any specific compiler flags or include external dependencies , then the needed command would be even more specific .";

fn main() {
    let map: HashMap<&str, Vec<usize>> = index_words(LARGE_TEXT);
    println!("{map:?}");
}

fn index_words(text: &str) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut i = 0;
    for word in text.split_whitespace() {
        map.entry(word).or_default().push(i);
        i += word.len() + 1;
    }
    return map;
}
