use config::Config;
extern crate futures;
use indicatif::{ProgressBar};
use futures::{ stream, StreamExt};
use reqwest::{Client as http};
use std::time::Duration;
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::ops::Not;
use colored::Colorize;
use crate::args::Args;
use clap::Parser;

fn add_extensions(wordlist: &mut String, words: &String, extensions: Vec<&str>) {
    eprintln!("Generating wordlist using ./wordlists/raft-small-words.txt and extensions: {}", extensions.connect(","));
    for e in extensions {
        wordlist.push_str(words.replace("\n", format!(".{}\n", e).as_str()).as_str());
    }
}

pub(crate) fn sort_wordlist(wordlist: &String) -> HashSet<String> {
    wordlist.lines()
        .map(|a|a.to_owned())
        .collect::<HashSet<String>>()
}

