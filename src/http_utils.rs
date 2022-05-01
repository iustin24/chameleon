use wappalyzer::Analysis;
use std::collections::{HashSet};
use std::ops::Not;
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use url::Url;
use futures::{ stream, StreamExt};
use reqwest::{Client as http};
use colored::Colorize;

#[derive(Debug)]
struct Data {
    path: String,
    http_code: String,
    location: String,
    http_size: String
}

pub(crate) async fn tech_detect(url: &str) -> Analysis {
    eprintln!("{}", format!("Started scanning {}\n", url).bold().green());
    let url = Url::parse(&url).unwrap();
    wappalyzer::scan(url).await
}

pub(crate) async fn http(paths: HashSet<String>, url: String, concurrency: u16) {
    let now = Instant::now();
    let bar = ProgressBar::new(paths.len() as u64);
    eprintln!("Probing {:?} urls", bar.length());
    let client_builder = http::builder().connect_timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36");
    let client = client_builder.build().unwrap();

    stream::iter(paths)
        .map(|path| async {
            bar.inc(1);
            let response = (&client).get(format!("{}{}", url, path).as_str()).send().await;
            (path, response)
        })
        .buffer_unordered(concurrency as usize)
        .filter_map(|(path, response)| async {
            let r = response.ok()?;

            let http_code = r.status().to_string()[0..3].to_owned();

            let location = if r.headers().contains_key("Location") {
                r.headers()["Location"].to_str().ok()?.to_string()
            } else {
                String::from("")
            };

            let http_size = match r.content_length() {
                Some(t) => t.to_string(),
                None => r.bytes().await.unwrap().len().to_string()
            };
            let data = Data {
                path,
                http_code,
                location,
                http_size
            };
            output(&data, vec!["404".to_owned(),"400".to_owned(),"302".to_owned()], bar.clone());
            Some(0)
        }).collect::<Vec<i32>>().await;
    println!("Total time elapsed: {}ms", now.elapsed().as_millis());
}

fn output(data: &Data, filter_codes: Vec<String>, bar: ProgressBar) {
    if filter_codes.contains(&data.http_code).not() {
        bar.println(format!("{0: <4} - {1: >7}B - /{2: <0} {3: <0}",
                            match &data.http_code.chars().nth(0) {
                                Some('2') => format!("{}", &data.http_code).green(),
                                Some('3') => format!("{}", &data.http_code).blue(),
                                Some('4') => format!("{}", &data.http_code).yellow(),
                                Some('5') => format!("{}", &data.http_code).red(),
                                _ => format!("{}", &data.http_code).white() },
                            data.http_size,
                            data.path,
                            if data.location.is_empty() {
                                String::from("")
                            } else {
                                format!("-> {}" , data.location)
                            }));
    }
}