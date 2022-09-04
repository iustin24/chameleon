use crate::Args;
use clap::Parser;
use colored::Colorize;
use futures::{stream, StreamExt};
use indicatif::ProgressBar;
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashSet;
use std::time::Instant;
use url::Url;
use wappalyzer::Analysis;
//use std::process::Command;

#[derive(Debug)]
struct Data {
    path: String,
    http_code: String,
    location: String,
    http_size: i64,
}

pub(crate) async fn tech_detect(url: &str) -> Analysis {
    eprintln!("{}", format!("Started scanning {}\n", url).bold().green());
    let url = Url::parse(&url).unwrap();
    wappalyzer::scan(url).await
}

pub(crate) async fn http(paths: HashSet<String>, args: &Args) {
    eprintln!("Probing {:?} urls", bar.length());

    let now = Instant::now();
    let bar = ProgressBar::new(paths.len() as u64);
    let client = args.build_client();
    let url = args.url.trim_end_matches("/");
    let parse = Url::parse(&url).unwrap();
    let path_prefix = parse.path();

    // Parse Filters
    let mut filter_sizes: Vec<&str> = vec![];
    if let Some(fs) = &args.filtersize {
        filter_sizes = fs.split(",").collect();
    }
    let filter_codes = &args.filtercode.split(",").collect::<Vec<&str>>();

    stream::iter(paths)
        .map(|path| async {
            bar.inc(1);
            let response = (&client)
                .get(format!("{}/{}", url, path).as_str())
                .send()
                .await;
            (path, response)
        })
        .buffer_unordered(args.concurrency as usize)
        .filter_map(|(path, response)| async {
            let r = response.ok()?;

            let http_code = r.status().to_string()[0..3].to_owned();
            if filter_codes.contains(&&**&http_code) {
                return None;
            }

            let location = if r.headers().contains_key("Location") {
                r.headers()["Location"].to_str().ok()?.to_string()
            } else {
                String::from("")
            };

            let http_size = match r.content_length() {
                Some(t) => t as i64,
                None => match r.bytes().await {
                    Ok(t) => t.len() as i64,
                    Err(_) => return None,
                },
            };
            if filter_sizes.contains(&&**&human_size(http_size)) {
                return None;
            }

            let data = Data {
                path,
                http_code,
                location,
                http_size,
            };
            output(&data, path_prefix, &bar);
            Some(0)
        })
        .collect::<Vec<i32>>()
        .await;
    eprintln!("Total time elapsed: {}ms", now.elapsed().as_millis());
}

pub(crate) fn add_extensions(wordlist: &mut String, words: &String, extensions: Vec<&str>) {
    eprintln!(
        "{}",
        format!(
            "Generating wordlist using ./wordlists/raft-small-words.txt and extensions: {}\n",
            extensions.join(",")
        )
        .bold()
    );
    for e in extensions {
        wordlist.push_str(words.replace("\n", format!(".{}\n", e).as_str()).as_str());
    }
}

pub(crate) fn sort_wordlist(wordlist: &String, iis: bool) -> HashSet<String> {
    match iis {
        true => {
            eprintln!(
                "{}",
                format!("Detected IIS - Using a lowercase only wordlist.\n").bold()
            );
            wordlist
                .lines()
                .map(|a| a.to_lowercase())
                .collect::<HashSet<String>>()
        }
        false => wordlist
            .lines()
            .map(|a| a.to_owned())
            .collect::<HashSet<String>>(),
    }
}

fn human_size(mut size: i64) -> String {
    let base = 1024;
    for unit in vec!["B", "KB", "MB", "GB"] {
        if (-1024 < size) && (size < 1024) {
            return format!("{}{}", size, unit);
        } else {
            size = size / base;
        }
    }
    format!("{}{}", size, "TB")
}

fn output(data: &Data, prefix: &str, bar: &ProgressBar) {
    bar.println(format!(
        "{0: <4} - {1: >7} - {2: <0} {3: <0}",
        match &data.http_code.chars().nth(0) {
            Some('2') => format!("{}", &data.http_code).green(),
            Some('3') => format!("{}", &data.http_code).blue(),
            Some('4') => format!("{}", &data.http_code).yellow(),
            Some('5') => format!("{}", &data.http_code).red(),
            _ => format!("{}", &data.http_code).white(),
        },
        human_size(data.http_size),
        format!("{}{}", prefix, data.path),
        if data.location.is_empty() {
            String::from("")
        } else {
            format!("-> {}", data.location)
        }
    ));
}
