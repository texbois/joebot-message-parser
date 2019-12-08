use chrono::NaiveDateTime;
use clap::{App, Arg};
use std::io::Write;
use vkopt_message_parser::filter::Filter;
use vkopt_message_parser::reader::{fold_html, EventResult, MessageEvent};

fn main() {
    let matches = App::new("VkOpt Message Parser")
        .args(&[
            Arg::with_name("only-include-names")
                .long("only-include-names")
                .help("Filter: screen names (id...) whose messages are included")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .conflicts_with("exclude-names"),
            Arg::with_name("exclude-names")
                .long("exclude-names")
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
            Arg::with_name("text-delimiter")
                .long("text-delimiter")
                .help("Delimiter inserted between messages (newline by default)")
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

    let output = matches.value_of("output").unwrap();
    let inputs = matches
        .values_of("inputs")
        .map(|ins| ins.collect())
        .unwrap();

    let delimiter = matches.value_of("text-delimiter").unwrap_or("\n");

    let short_name_whitelist = matches
        .values_of("only-include-names")
        .map(|ns| ns.collect());
    let short_name_blacklist = matches.values_of("exclude-names").map(|ns| ns.collect());
    let since_date = matches
        .value_of("since-date")
        .map(|d| NaiveDateTime::parse_from_str(d, "%Y.%m.%d %H:%M:%S").unwrap());
    let filter = Filter {
        short_name_whitelist,
        short_name_blacklist,
        since_date,
    };

    write(inputs, output, &filter, delimiter).unwrap();
}

fn write<'w>(
    inputs: Vec<&'w str>,
    output: &'w str,
    filter: &Filter<'w>,
    delimiter: &'w str,
) -> quick_xml::Result<()>
{
    let folded: quick_xml::Result<Vec<String>> = inputs
        .iter()
        .map(|i| {
            fold_html(i, String::new(), |mut acc, event| {
                match filter.filter_event(event) {
                    Some(e) => match e {
                        MessageEvent::BodyExtracted(body) if !body.is_empty() => {
                            acc += &body;
                            acc += delimiter;
                            EventResult::Consumed(acc)
                        }
                        _ => EventResult::Consumed(acc),
                    },
                    None => EventResult::SkipMessage(acc),
                }
            })
        })
        .collect();

    let mut out = std::fs::File::create(output)?;
    for acc in folded?.iter() {
        write!(&mut out, "{}", acc)?;
    }
    Ok(())
}
