#[macro_use]
extern crate clap;

use chrono::NaiveDateTime;
use clap::{App, Arg};
use joebot_message_parser::filter::Filter;
use joebot_message_parser::writers::{TextWriter, Writer};

arg_enum! {
    #[derive(Debug)]
    enum WriterType { Taki, Text }
}

fn main() {
    let matches = App::new("Joebot Message Parser")
        .args(&[
            Arg::with_name("writer")
                .help("Output writer")
                .required(true)
                .takes_value(true)
                .case_insensitive(true)
                .possible_values(&WriterType::variants()),
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

    let writer_type = value_t!(matches.value_of("writer"), WriterType).unwrap_or_else(|e| e.exit());
    let output = matches.value_of("output").unwrap();
    let inputs = matches.values_of("inputs").map(|ins| ins.collect()).unwrap();

    let short_name_blacklist = matches.values_of("ignore-names").map(|ns| ns.collect());
    let since_date = matches
        .value_of("since-date")
        .map(|d| NaiveDateTime::parse_from_str(d, "%Y.%m.%d %H:%M:%S").unwrap());
    let filter = Filter {
        short_name_blacklist,
        since_date,
    };

    match writer_type {
        WriterType::Text => <TextWriter as Writer>::write(inputs, output, &filter).unwrap(),
        _ => unimplemented!()
    };
}
