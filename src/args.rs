extern crate dirs;
use anyhow::Result;
use clap::Parser;
use config::Config;
use reqwest::{Client as http, Client};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::time::Duration;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum StatusCodes {
    All,
    Codes(Vec<u16>),
}


impl FromStr for StatusCodes {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "all" {
            Ok(StatusCodes::All)
        } else {
            let codes: Vec<u16> = s
                .split(',')
                .map(|code| code.parse::<u16>().map_err(|_| format!("Invalid code: {}", code)))
                .collect::<Result<Vec<u16>, String>>()?;
            Ok(StatusCodes::Codes(codes))
        }
    }
}

#[derive(Debug)]
pub enum Size {
    Single(usize),
    Range(usize, usize),
}

impl FromStr for Size {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();

        match parts.len() {
            1 => Ok(Size::Single(parts[0].parse::<usize>().map_err(|e: std::num::ParseIntError| e.to_string())?)),
            2 => Ok(Size::Range(
                parts[0].parse::<usize>().map_err(|e: std::num::ParseIntError| e.to_string())?,
                      parts[1].parse::<usize>().map_err(|e: std::num::ParseIntError| e.to_string())?
            )),
            _ => Err("Invalid filter size format".to_string()),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(
        short = 'w',
        long = "wordlist",
        help = "Main wordlist to use for bruteforcing"
    )]
    pub(crate) wordlist: Option<String>,

    #[clap(
        short = 'W',
        long = "small-wordlist",
        help = "Wordlist used to generate files by adding extensions ( FUZZ.%ext )"
    )]
    pub(crate) small_wordlist: Option<String>,

    #[clap(short = 'L', long = "hosts-file", help = "List of hosts to scan")]
    pub(crate) hosts_file: Option<String>,

    #[clap(
        short = 'a',
        long = "tech-detect",
        help = "Automatically detect technologies with wappalyzer and adapt wordlist",
        takes_value = false
    )]
    pub(crate) tech_detect: bool,

    #[clap(
        short = 'A',
        long = "auto-calibrate",
        help = "Automatically calibrate filtering options (default: false)",
        takes_value = false
    )]
    pub(crate) auto_calibrate: bool,

    #[clap(
        short = 'k',
        long = "config",
        help = "Config file to use",
        default_value = "~/.config/chameleon/config.toml"
    )]
    pub(crate) config: String,

    #[clap(short = 'u', long = "url", help = "url to scan")]
    pub(crate) url: Option<String>,
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
        default_value = "40"
    )]
    pub(crate) concurrency: usize,

    #[clap(
        short = 'T',
        long = "tech url",
        help = "URL which will be scanned for technologies. By default, this is the same as '-u', however it can be changed using '-T'"
    )]
    pub(crate) tech_url: Option<String>,

    #[clap(
        short = 'i',
        long = "include tech",
        help = "Technology to be included, even if its not detected by wappalyzer. ( -i PHP,IIS )"
    )]
    pub(crate) techs: Option<String>,

    #[clap(
        short = 'S',
        long = "fs",
        help = "Filter HTTP response size. Comma separated list of sizes",
        multiple = true,
        use_value_delimiter = true,
        value_delimiter = ',',
        parse(try_from_str = Size::from_str)
    )]
    pub(crate) filtersize: Option<Vec<Size>>,

    #[clap(
        short = 'C',
        long = "fc",
        help = "Filter HTTP status codes from response - Comma separated list",
        parse(try_from_str),
    )]
    pub(crate) filtercode: Option<StatusCodes>,

    #[clap(
        short = 's',
        long = "ms",
        help = "Match HTTP response size. Comma separated list of sizes",
        multiple = true,
        use_value_delimiter = true,
        value_delimiter = ',',
        parse(try_from_str = Size::from_str)
    )]
    pub(crate) matchsize: Option<Vec<Size>>,

        #[clap(
        short = 'c',
        long = "mc",
        help = "Match HTTP status codes from response - Comma separated list ('all' for all codes 0-999)",
        parse(try_from_str),
        default_value = "200,204,301,302,307,401,403,405"
    )]
    pub(crate) matchcode: StatusCodes,

    #[clap(
        short = 'U',
        long = "user-agent",
        help = "Change the value for the user-agent header",
        default_value = "Chameleon / https://github.com/iustin24/chameleon"
    )]
    pub(crate) useragent: String,

    #[clap(short = 'o', long = "output", help = "Save the output into a file")]
    pub(crate) output: Option<String>,

    #[clap(
        short = 'J',
        long = "json",
        help = "Save the output as json",
        takes_value = false
    )]
    pub(crate) json: bool,

    #[clap(
        short = 'X',
        long = "methods",
        help = "HTTP Methods to use. Comma separated list of sizes",
        default_value = "GET",
        multiple = true,
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    pub(crate) methods: Vec<String>,

    #[clap(
        short = 'z',
        long = "timeout",
        help = "Request timeout length ( default: 10 )",
        default_value = "10"
    )]
    pub(crate) timeout: u64,
}

impl Args {
    pub(crate) fn get_main_wordlist_str(
        &self,
        settings: &HashMap<String, String>,
    ) -> Result<String> {
        let output = match self.wordlist {
            Some(ref path) => read_to_string(path)?,
            None => read_to_string(self.get_wordlist_path(settings, "main_wordlist").unwrap())?,
        };
        Ok(output)
    }

    pub(crate) fn build_client(&self) -> Client {
        let client_builder = http::builder()
            .connect_timeout(Duration::from_secs(self.timeout))
            .danger_accept_invalid_certs(true)
            .redirect(reqwest::redirect::Policy::none())
            //.proxy(reqwest::Proxy::https("http://127.0.0.1:8080").unwrap())
            .timeout(Duration::from_secs(self.timeout))
            .user_agent(&self.useragent);
        client_builder.build().unwrap()
    }

    pub(crate) fn get_small_wordlist_str(
        &self,
        settings: &HashMap<String, String>,
    ) -> Result<String> {
        let output = match self.small_wordlist {
            Some(ref path) => read_to_string(path)?,
            None => read_to_string(self.get_wordlist_path(settings, "small_wordlist").unwrap())?,
        };
        Ok(output)
    }

    pub(crate) fn get_wordlist_path<'a>(
        &self,
        settings: &'a HashMap<String, String>,
        wordlist: &str,
    ) -> Option<&'a String> {
        settings.get(&wordlist.replace(" ", "_").replace(".", "_"))
    }

    pub(crate) fn get_extensions<'a>(
        &self,
        settings: &'a HashMap<String, String>,
        wordlist: &str,
    ) -> Option<&'a String> {
        settings.get(&(wordlist.replace(" ", "_").replace(".", "_") + "_ext"))
    }

    pub(crate) fn get_config(&self) -> HashMap<String, String> {
        Config::builder()
            .add_source(config::File::with_name(
                &self
                    .config
                    .as_str()
                    .replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
            ))
            .build()
            .unwrap()
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
            .into_iter()
            .map(|(key, value)| {
                (
                    key,
                    value.replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
                )
            })
            .collect()
    }
}
