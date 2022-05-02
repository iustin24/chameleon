use std::collections::{HashSet};

fn add_extensions(wordlist: &mut String, words: &String, extensions: Vec<&str>) {
    eprintln!("Generating wordlist using ./wordlists/raft-small-words.txt and extensions: {}", extensions.join(","));
    for e in extensions {
        wordlist.push_str(words.replace("\n", format!(".{}\n", e).as_str()).as_str());
    }
}

pub(crate) fn sort_wordlist(wordlist: &String) -> HashSet<String> {
    wordlist.lines()
        .map(|a|a.to_owned())
        .collect::<HashSet<String>>()
}

