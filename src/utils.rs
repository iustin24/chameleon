mod decider;
use crate::utils::decider::FilterDecider;
use crate::Args;
use colored::Colorize;
use feroxfuzz::client::AsyncClient;
use feroxfuzz::corpora::Wordlist;
use feroxfuzz::deciders::LogicOperation;
use feroxfuzz::fuzzers::AsyncFuzzer;
use feroxfuzz::mutators::ReplaceKeyword;
use feroxfuzz::observers::ResponseObserver;
use feroxfuzz::prelude::*;
use feroxfuzz::processors::ResponseProcessor;
use feroxfuzz::responses::AsyncResponse;
use feroxfuzz::responses::Response;
use feroxfuzz::schedulers::OrderedScheduler;
use indicatif::ProgressBar;
use std::collections::HashSet;
use std::time::Instant;
use url::Url;
use wappalyzer::Analysis;

pub(crate) async fn tech_detect(url: &str) -> Analysis {
    eprintln!("{}", format!("Started scanning {}\n", url).bold().green());
    let url = Url::parse(&url).unwrap();
    wappalyzer::scan(url).await
}

pub(crate) async fn http(paths: HashSet<String>, args: &Args) {
    let bar = ProgressBar::new(paths.len() as u64);
    let words = Wordlist::with_words(paths).name("words").build();
    let mut state = SharedState::with_corpus(words);
    let now = Instant::now();
    eprintln!("Probing {:?} urls", bar.length());

    let client = args.build_client();
    let async_client = AsyncClient::with_client(client);

    let mutator = ReplaceKeyword::new(&"FUZZ", "words");
    let parse = Url::parse(args.url.as_str()).expect("Invalid URL");
    let request = Request::from_url(
        args.url.as_str(),
        Some(&[ShouldFuzz::URLPath(
            format!("{}FUZZ", parse.path()).as_ref(),
        )]),
    )
    .unwrap();
    let response_observer: ResponseObserver<AsyncResponse> = ResponseObserver::new();

    let code_decider = FilterDecider::new(args, |args, code, length, _state| {
        let mut action = match &args.matchcode {
            Some(mc) => filter(mc, &code, true),
            _ => Action::Keep
        };
        if action == Action::Keep {
            match &args.matchsize {
                Some(ms) => action = filter(ms, &length, true),
                _ => ()
            }
        }
        action
    });


    let length_decider = FilterDecider::new(args, |args, code, length, _state| {
        let mut action = filter(&args.filtercode, &code, false);
        if action == Action::Keep {
            match &args.filtersize {
                Some(fs) => action = filter(fs, &length, false),
                _ => ()
            }
        }
        action
    });

    let response_printer = ResponseProcessor::new(
        |response_observer: &ResponseObserver<AsyncResponse>, action, _state| {
            bar.inc(1);
            if let Some(Action::Keep) = action {
                bar.println(format!(
                    "{0: <4} - {1: >7}B - {2: <0} {3: <0}",
                    match response_observer.status_code().to_string().chars().nth(0) {
                        Some('2') => format!("{}", &response_observer.status_code()).green(),
                        Some('3') => format!("{}", &response_observer.status_code()).blue(),
                        Some('4') => format!("{}", &response_observer.status_code()).yellow(),
                        Some('5') => format!("{}", &response_observer.status_code()).red(),
                        _ => format!("{}", &response_observer.status_code()).white(),
                    },
                    response_observer.content_length(),
                    response_observer.url().path(),
                    match (
                        response_observer.headers().get("location"),
                        response_observer.headers().get("Location")
                    ) {
                        (Some(location), _) | (_, Some(location)) =>
                            format!("-> {}", String::from_utf8_lossy(location).to_string()),
                        _ => String::from(""),
                    }
                ));
            }
        },
    );

    let scheduler = OrderedScheduler::new(state.clone()).unwrap();
    let deciders = build_deciders!(code_decider, length_decider);
    let mutators = build_mutators!(mutator);
    let observers = build_observers!(response_observer);
    let processors = build_processors!(response_printer);

    let threads = args.concurrency;

    let mut fuzzer = AsyncFuzzer::new(
        threads,
        async_client,
        request,
        scheduler,
        mutators,
        observers,
        processors,
        deciders,
    );

    fuzzer.set_post_send_logic(LogicOperation::And);

    fuzzer.fuzz_once(&mut state).await.unwrap();
    //println!("{state:#}");
    eprintln!("Total time elapsed: {}ms", now.elapsed().as_millis());
}

pub(crate) fn add_extensions(wordlist: &mut String, words: &String, extensions: Vec<&str>) {
    eprintln!(
        "{}",
        format!(
            "Generating wordlist using supplied small wordlist and extensions: {}\n",
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

fn filter<T: PartialEq>(vec: &Vec<T>, c: &T, m: bool) -> Action {
    if vec.contains(c) {
        if m {
            Action::Keep
        } else {
            Action::Discard
        }
    } else {
        if m {
            Action::Discard
        } else {
            Action::Keep
        }
    }
}