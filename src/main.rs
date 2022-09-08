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

    let tech_url = match &args.tech_url {
        Some(url) => url,
        _ => &args.url,
    };

    // Detect Technologies running on the host
    let mut tech: Vec<Tech> = vec![];
    if args.tech_detect {
        match utils::tech_detect(tech_url.as_str()).await.result {
            Ok(t) => tech = t,
            Err(e) => eprintln!(
                "Failed to detect technologies . Got following error: {} ",
                e
            ),
        };
    }

    // Import wordlists specific to detected technologies
    if let Some(techs) = &args.techs {
        techs
            .split(",")
            .filter_map(|t| {
                for tt in &tech {
                    if tt.name.eq(t) {
                        return None;
                    }
                }
                tech.push(Tech {
                    name: String::from(t),
                    category: String::from(""),
                });
                Some(0)
            })
            .for_each(drop);
    }

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
                &args.get_small_wordlist_str(&settings).unwrap(),
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
        &args,
    )
    .await;
}
