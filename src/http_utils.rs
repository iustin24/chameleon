use crate::{args, Args};
use wappalyzer::Analysis;
use std::collections::{HashSet};
use std::ops::Not;
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use url::Url;
use futures::{ stream, StreamExt};
use reqwest::{Client as http};
use colored::Colorize;
use clap::Parser;

#[derive(Debug)]
struct Data {
    path: String,
    http_code: String,
    location: String,
    http_size: i64
}

pub(crate) async fn tech_detect(url: &str) -> Analysis {
    eprintln!("{}", format!("Started scanning {}\n", url).bold().green());
    let url = Url::parse(&url).unwrap();
    wappalyzer::scan(url).await
}

pub(crate) async fn http(paths: HashSet<String>, url: String) {
    let now = Instant::now();
    let args: Args = Args::parse();
    let bar = ProgressBar::new(paths.len() as u64);
    eprintln!("Probing {:?} urls", bar.length());
    let client = http::builder().connect_timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36")
        .build().unwrap();
    let path_prefix = Url::parse(&url).unwrap();
    let url = url.trim_end_matches("/");
    stream::iter(paths)
        .map(|path| async {
            bar.inc(1);
            let response = (&client).get(format!("{}/{}", url, path).as_str()).send().await;
            (path, response)
        })
        .buffer_unordered(args.concurrency as usize)
        .filter_map(|(path, response)| async {
            let r = response.ok()?;

            let http_code = r.status().to_string()[0..3].to_owned();

            let location = if r.headers().contains_key("Location") {
                r.headers()["Location"].to_str().ok()?.to_string()
            } else {
                String::from("")
            };

            let http_size = match r.content_length() {
                Some(t) => t as i64,
                None => r.bytes().await.unwrap().len() as i64
            };
            let data = Data {
                path,
                http_code,
                location,
                http_size
            };
            output(&data, &path_prefix.path(), bar.clone());
            Some(0)
        }).collect::<Vec<i32>>().await;
    println!("Total time elapsed: {}ms", now.elapsed().as_millis());
}

fn human_size(mut size: i64) -> String {
    let base = 1024;
    for unit in vec!["B ", "KB", "MB", "GB"] {
        if (-1024 < size) && ( size < 1024) {
           return format!("{}{}", size, unit);
        } else {
            size = size / base;
        }
    }
    format!("{}{}", size, "TB")
}

fn filter(data: &Data) -> Option<&Data>{
    let args: Args = Args::parse();
    match args.filtersize {
        Some(t) => {
            for size in t.split(",") {
                if human_size(data.http_size).eq(size) {
                    return None;
                }
            }
        }
        None => ()
    }
    for code in args.filtercode.split(",") {
       if data.http_code.eq(code) {
           return None;
       }
    }
    Some(data)
}

fn output(data: &Data, prefix: &str, bar: ProgressBar) {
    match filter(data) {
        Some(data) => {
            bar.println(format!("{0: <4} - {1: >7} - {2: <0} {3: <0}",
                                match &data.http_code.chars().nth(0) {
                                    Some('2') => format!("{}", &data.http_code).green(),
                                    Some('3') => format!("{}", &data.http_code).blue(),
                                    Some('4') => format!("{}", &data.http_code).yellow(),
                                    Some('5') => format!("{}", &data.http_code).red(),
                                    _ => format!("{}", &data.http_code).white() },
                                human_size(data.http_size),
                                format!("{}{}", prefix ,data.path),
                                if data.location.is_empty() {
                                    String::from("")
                                } else {
                                    format!("-> {}" , data.location)
                                }));

        }
        None => ()
    }
}