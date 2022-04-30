mod args;

use config::Config;
extern crate futures;
use indicatif::{ProgressBar};
use futures::{ stream, StreamExt};
use reqwest::{Client as http};
use std::time::Duration;
use regex::Regex;
use lazy_static::lazy_static;
use url::{Url};
use std::time::Instant;
use wappalyzer::Analysis;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::ops::Not;
use colored::Colorize;
use crate::args::Args;
use clap::Parser;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?im)<\s*title.*>(.*?)<\s*/\s*title>").unwrap();
}

#[derive(Debug)]
struct Data {
    path: String,
    http_code: String,
    http_title: String,
    location: String,
}

async fn tech_detect(url: &str) -> Analysis {
    let url = Url::parse(&url).unwrap();
    wappalyzer::scan(url).await
}

async fn http(paths: HashSet<String>, url: String) {
    let now = Instant::now();
    let bar = ProgressBar::new(paths.len() as u64);
    println!("Probing {:?} urls", bar.length());
    let client_builder = http::builder().connect_timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36");
    let client = client_builder.build().unwrap();
    let results = stream::iter(paths)
        .map(|path| async {
            bar.inc(1);
            let response = (&client).get(format!("{}{}", url, path).as_str()).send().await;
            (path, response)
        })
        .buffer_unordered(200 )
        .filter_map(|(url, response)| async {
            let r = response.ok()?;

            let mut title= String::from("");

            let code = &r.status().to_string()[0..3];

            let location = if r.headers().contains_key("Location") { r.headers()["Location"].to_str().ok()?.to_string()} else { String::from("") };

            match r.headers().get("content-type") {
                Some(header) => {
                    if header.to_str().unwrap_or("").contains("text/html"){
                        let body = r
                            .text()
                            .await
                            .ok()?;
                        for cap in RE.captures_iter(&body).nth(0) {
                            title = cap[1].to_string();
                        }
                    }
                },
                None => ()
            }

            let data = Data {
                path: url,
                http_code: code.to_string(),
                http_title: title.chars().take(254).collect(),
                location: location.to_string().chars().take(1024).collect(),
            };
            output(&data, vec!["404".to_owned(),"400".to_owned(),"302".to_owned()], bar.clone());
            Some(data)
        }).collect::<Vec<Data>>().await;
    println!("Total time elapsed: {}ms", now.elapsed().as_millis());
}

fn add_extensions(wordlist: &mut String, words: &String, extensions: Vec<&str>) {
    eprintln!("Generating wordlist using ./wordlists/raft-small-words.txt and extensions: {}", extensions.connect(","));
    for e in extensions {
        wordlist.push_str(words.replace("\n", format!(".{}\n", e).as_str()).as_str());
    }
}

fn sort_wordlist(wordlist: &String) -> HashSet<String> {
    wordlist.lines()
        .map(|a|a.to_owned())
        .collect::<HashSet<String>>()
}

fn output(data: &Data, filter_codes: Vec<String>, bar: ProgressBar) {
    if filter_codes.contains(&data.http_code).not() {
        bar.println(format!("{} - /{} {}",
                            match &data.http_code.chars().nth(0) {
                                Some('2') => format!("{}", &data.http_code).green(),
                                Some('3') => format!("{}", &data.http_code).blue(),
                                Some('4') => format!("{}", &data.http_code).yellow(),
                                Some('5') => format!("{}", &data.http_code).red(),
                                _ => format!("{}", &data.http_code).white() },
                            data.path,
                            if data.location.is_empty() {
                                String::from("")
                            } else {
                                format!("-> {}" , data.location)
                            }));
    }

}

#[tokio::main]
async fn main() {

    let settings = Config::builder()
        .add_source(config::File::with_name("./config"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();
    let url = "https://gold.zebra.com/";
    let tech = tech_detect(url).await;
    eprintln!("{}", format!("Started scanning {}\n", url).bold().green());
    let mut wordlist = read_to_string(settings.get("main_wordlist").expect("Invalid Config File"))
        .expect("Unable to read wordlist.");

    let small_words = read_to_string(settings.get("small_wordlist").expect("Invalid Config File"))
        .expect("Unable to read wordlist.");

    for fg in tech.result.unwrap() {
        match settings.get(fg.name
            .as_str()
            .replace(" ", "_")
            .replace(".", "_")
            .as_str()) {
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

    http(sort_wordlist(&wordlist), url.to_owned()).await;

}