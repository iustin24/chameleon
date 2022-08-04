mod args;
mod utils;

extern crate dirs;
use crate::args::Args;
use clap::Parser;
use colored::Colorize;
use std::fs::read_to_string;
use wappalyzer::wapp::Tech;

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let settings = args.get_config();

    let mut main_wordlist = args.get_main_wordlist_str(&settings).unwrap();

    let small_wordlist = args.get_small_wordlist_str(&settings).unwrap();

    let tech_url = match &args.tech_url {
        Some(url) => url,
        _ => &args.url,
    };

    // Detect Technologies running on the host
    let tech = utils::tech_detect(tech_url.as_str()).await.result.unwrap();
    // Import wordlists specific to detected technologies
    for fg in &tech {
        match args.get_wordlist_path(&settings, &fg.name) {
            Some(wordlist_path) => {
                eprintln!(
                    "{}",
                    format!("Detected Technology - {} ( {} )\n", fg.name, wordlist_path).bold()
                );

                main_wordlist.push_str(
                    read_to_string(wordlist_path)
                        .expect("Unable to read wordlist.")
                        .as_str(),
                );
            }
            None => (),
        }

        match args.get_extensions(&settings, &fg.name) {
            Some(t) => utils::add_extensions(
                &mut main_wordlist,
                &small_wordlist,
                t.split(",").collect::<Vec<&str>>(),
            ),
            None => (),
        }
    }

    // Start Bruteforcing
    utils::http(
        utils::sort_wordlist(
            &main_wordlist,
            tech.contains(&Tech {
                name: String::from("IIS"),
                category: String::from("Web servers"),
            }) || tech.contains(&Tech {
                name: String::from("Microsoft ASP.NET"),
                category: String::from("Web Application Frameworks"),
            }),
        ),
        args.url,
    )
    .await;
}
