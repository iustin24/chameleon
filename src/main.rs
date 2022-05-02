mod args;
mod http_utils;
mod utils;

extern crate futures;
extern crate dirs;
use std::fs::read_to_string;
use crate::args::Args;
use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() {

    let args: Args = Args::parse();

    let mut wordlist = args.get_wordlist_str()
        .unwrap();

    let settings = args.get_config();

    let tech = http_utils::tech_detect(args.url.as_str()).await;

    for fg in tech.result.unwrap() {
        match args.get_wordlist_path(&settings, &fg.name) {
            Some(wordlist_path) => {
                eprintln!("{}", format!("Detected Technology - {} ( {} )\n", fg.name, wordlist_path).bold());
                wordlist.push_str(read_to_string(wordlist_path)
                    .expect("Unable to read wordlist.")
                    .as_str()
                );
            },
            None => ()
        }
    }

    http_utils::http(utils::sort_wordlist(&wordlist), args.url).await;

}