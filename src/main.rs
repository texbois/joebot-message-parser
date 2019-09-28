#[macro_use]
extern crate clap;

use chrono::NaiveDateTime;
use clap::{App, Arg};
use joebot_message_parser::filter::Filter;
use joebot_message_parser::reader::{fold_html, EventResult, MessageEvent};
use std::collections::BTreeSet;

arg_enum! {
    #[derive(Debug)]
    enum Writer { Taki, Text }
}

fn main() {
    let matches = App::new("Joebot Message Parser")
        .args(&[
            Arg::with_name("writer")
                .help("Output writer")
                .required(true)
                .takes_value(true)
                .case_insensitive(true)
                .possible_values(&Writer::variants()),
            Arg::with_name("ignore-names")
                .long("ignore-names")
                .help("Filter: screen names (id...) whose messages are excluded")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true),
            Arg::with_name("since-date")
                .long("since-date")
                .help("Filter: minimum date for a message to be included (ex: 2019.01.01 13:00:00)")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true),
            Arg::with_name("output")
                .short("o")
                .help("Output file path")
                .required(true)
                .takes_value(true),
            Arg::with_name("inputs")
                .help("Input files (.htmls exported using VkOpt)")
                .last(true)
                .required(true)
                .multiple(true)
                .takes_value(true),
        ])
        .get_matches();

    let writer = value_t!(matches.value_of("writer"), Writer).unwrap_or_else(|e| e.exit());
    let output_path = matches.value_of("output").unwrap();
    let inputs = matches.values_of("inputs").unwrap();

    let short_name_blacklist = matches
        .values_of("ignore-names")
        .map(|ns| ns.map(|n| n.to_owned()).collect::<BTreeSet<String>>());
    let since_date = matches
        .value_of("since-date")
        .map(|d| NaiveDateTime::parse_from_str(d, "%Y.%m.%d %H:%M:%S").unwrap());
    let filter = Filter {
        short_name_blacklist,
        since_date,
    };

    for input in inputs {
        let text = fold_html(input, String::new(), |mut acc, event| match event {
            MessageEvent::BodyExtracted(body) if !body.is_empty() => {
                acc += &body;
                acc += "\n";
                EventResult::Consumed(acc)
            }
            _ => EventResult::Consumed(acc),
        })
        .unwrap();
        std::fs::write(output_path, text).unwrap();
    }
}
