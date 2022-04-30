use std::fs::read_to_string;
use std::io::{BufRead, stdin};
use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(
    short = 'd',
    long = "domains",
    help = "The file containing domains you want to generate permutations from. If this is not specified, domains are read from stdin."
    )]
    pub(crate) domain_file_path: Option<String>,

    #[clap(
    short = 'w',
    long = "wordlist",
    help = "The supplementary wordlist file to include."
    )]
    pub(crate) wordlist: Option<String>,

    #[clap(
    short = 'l',
    long = "len",
    help = "The minimum length for a word to be considered important. If not specified, all words are accepted."
    )]
    pub(crate) min_word_len: Option<usize>,
}

impl Args {
    pub(crate) fn get_wordlist_str(&self) -> Result<String> {
        let output = match self.wordlist {
            Some(ref path) => read_to_string(path)?,
            None => String::new()
        };

        Ok(output)
    }
}