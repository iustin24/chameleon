extern crate dirs;
use std::collections::{HashMap};
use std::fs::read_to_string;
use std::time::Duration;
use clap::Parser;
use anyhow::Result;
use config::{Config};
use reqwest::{Client as http, Client};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(
    short = 'w',
    long = "wordlist",
    help = "Main wordlist to use for bruteforcing",
    )]
    pub(crate) wordlist: Option<String>,

    #[clap(
    short = 'W',
    long = "small wordlist",
    help = "Wordlist used to generate files by adding extensions ( FUZZ.%ext )",
    )]
    pub(crate) small_wordlist: Option<String>,

    /*
    #[clap(
    short = 'd',
    long = "download-wordlists",
    help = "Download custom Wordlists",
    takes_value = false,
    )]
    pub(crate) download: Option<String>,
    */

    #[clap(
    short = 'c',
    long = "config",
    help = "Config file to use",
    default_value = "~/.config/chameleon/config.toml"
    )]
    pub(crate) config: String,

    #[clap(
    short = 'u',
    long = "url",
    help = "url to scan"
    )]
    pub(crate) url: String,
/*
    #[clap(
    short = 'H',
    long = "HTTP Header",
    help = "HTTP header. Multiple -H flags are accepted."
    )]
    pub(crate) header: Option<String>,
*/
    #[clap(
    short = 't',
    long = "concurrency",
    help = "Number of concurrent threads ( default: 200 )",
    default_value = "200"
    )]
    pub(crate) concurrency: u16,

    #[clap(
    short = 'T',
    long = "tech url",
    help = "URL which will be scanned for technologies. By default, this is the same as '-u', however it can be changed using '-T'"
    )]
    pub(crate) tech_url: Option<String>,

    #[clap(
    short = 'S',
    long = "fs",
    help = "Filter HTTP response size. Comma separated list of sizes and ranges",
    )]
    pub(crate) filtersize: Option<String>,

    #[clap(
    short = 'f',
    long = "fc",
    help = "Filter HTTP status codes from response - Comma separated list",
    default_value = "404"
    )]
    pub(crate) filtercode: String,
}

impl Args {
    pub(crate) fn get_main_wordlist_str(&self, settings: &HashMap<String, String>) -> Result<String> {
        let output = match self.wordlist {
            Some(ref path) => read_to_string(path)?,
            None => {
                read_to_string(self.get_wordlist_path(settings, "main_wordlist").unwrap())?
            }
        };
        Ok(output)
    }
    pub(crate) fn build_client(&self) -> Client {
        let client_builder = http::builder().connect_timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36");
        client_builder.build().unwrap()
    }

    pub(crate) fn get_small_wordlist_str(&self, settings: &HashMap<String, String>) -> Result<String> {
        let output = match self.small_wordlist {
            Some(ref path) => read_to_string(path)?,
            None => {
                read_to_string(self.get_wordlist_path(settings, "small_wordlist").unwrap())?
            }
        };
        Ok(output)
    }

    pub(crate) fn get_wordlist_path<'a>(&self, settings: &'a HashMap<String, String>, wordlist: &str) -> Option<&'a String> {
        settings.get(&wordlist
                .replace(" ", "_")
                .replace(".", "_")
            )
    }

    pub(crate) fn get_extensions<'a>(&self, settings: &'a HashMap<String, String>, wordlist: &str) -> Option<&'a String> {
        settings.get(&( wordlist
            .replace(" ", "_")
            .replace(".", "_") + "_ext")
        )
    }

    pub(crate) fn get_config(&self) -> HashMap<String, String> {
        Config::builder()
            .add_source(config::File::with_name(&self.config.as_str().replace("~", dirs::home_dir().unwrap().to_str().unwrap())))
            .build()
            .unwrap()
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
            .into_iter()
            .map(|(key, value)|
                (key, value.replace("~", dirs::home_dir().unwrap().to_str().unwrap()))
            )
            .collect()
    }
}