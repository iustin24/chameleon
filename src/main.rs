mod args;
mod http_utils;
mod utils;

extern crate dirs;
use std::fs::read_to_string;
use crate::args::Args;
use clap::Parser;
use colored::Colorize;
use wappalyzer::wapp::Tech;

#[tokio::main]
async fn main() {

    let args: Args = Args::parse();

    let settings = args.get_config();

    let mut main_wordlist = args.get_main_wordlist_str(&settings)
        .unwrap();

    let small_wordlist = args.get_small_wordlist_str(&settings)
        .unwrap();


    // Detect Technologies running on the host
    let tech = http_utils::tech_detect(args.url.as_str()).await.result.unwrap();

    // Import wordlists specific to detected technologies
    for fg in &tech {
        match args.get_wordlist_path(&settings, &fg.name) {
            Some(wordlist_path) => {

                eprintln!("{}", format!("Detected Technology - {} ( {} )\n", fg.name, wordlist_path).bold());

                main_wordlist.push_str(read_to_string(wordlist_path
                            .replace("~", dirs::home_dir() // replace tilde with home directory
                                .unwrap()
                                .to_str()
                                .unwrap()
                            ))
                    .expect("Unable to read wordlist.")
                    .as_str());

            },
            None => ()
        }

        match args.get_extensions(&settings, &fg.name) {
            Some(t) => http_utils::add_extensions(&mut main_wordlist, &small_wordlist, t.split(",").collect::<Vec<&str>>()) ,
            None => ()
        }
    }

    // Start Bruteforcing
    http_utils::http(
        http_utils::sort_wordlist(
            &main_wordlist,
            tech.contains(&Tech { name: String::from("IIS"), category: String::from("Web servers")})
            ),
        args.url)
    .await;
}