extern crate dirs;
use std::collections::{HashMap};
use std::fs::read_to_string;
use clap::Parser;
use anyhow::Result;
use config::{Config};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(
    short = 'w',
    long = "wordlist",
    help = "The supplementary wordlist file to include.",
    )]
    pub(crate) wordlist: Option<String>,

    #[clap(
    short = 'c',
    long = "config",
    help = "Config file to use",
    default_value = "~/.config/content/config.toml"
    )]
    pub(crate) config: String,

    #[clap(
    short = 'u',
    long = "url",
    help = "url to scan"
    )]
    pub(crate) url: String,

    #[clap(
    short = 't',
    long = "concurrency",
    help = "Number of concurrent threads ( default: 200 )",
    default_value = "200"
    )]
    pub(crate) concurrency: u16,

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
    pub(crate) fn get_wordlist_str(&self) -> Result<String> {
        let output = match self.wordlist {
            Some(ref path) => read_to_string(path)?,
            None => {
                read_to_string(self.get_wordlist_path(&self.get_config(),"main_wordlist").unwrap())?
            }
        };
        Ok(output)
    }
    pub(crate) fn get_wordlist_path(&self, settings: &HashMap<String, String>, wordlist: &str) -> Option<String> {
        match settings.get(&wordlist
                .replace(" ", "_")
                .replace(".", "_")
            ) {
            Some(t) => Some(t.replace("~", dirs::home_dir().unwrap().to_str().unwrap())),
            None => None
        }

    }
    pub(crate) fn get_config(&self) -> HashMap<String, String> {
        Config::builder()
            .add_source(config::File::with_name(&self.config.as_str().replace("~", dirs::home_dir().unwrap().to_str().unwrap())))
            .build()
            .unwrap()
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    }
}