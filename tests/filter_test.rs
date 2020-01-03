use vkopt_message_parser::filter::Filter;

mod test_helper;
use test_helper::*;

pub fn read_events_filtered(fixture: &str, filter: Filter) -> Vec<String> {
    read_events_skipping(fixture, |e| filter.filter_event(e).is_some())
}

#[test]
fn it_does_not_apply_filters_by_default() {
    let filtered = read_events_filtered("messages.html", Default::default());
    let unfiltered = read_events("messages.html");
    assert_eq!(filtered, unfiltered);
}

#[test]
fn it_filters_by_min_date() {
    let date =
        chrono::NaiveDateTime::parse_from_str("2018.01.22 00:00:00", "%Y.%m.%d %H:%M:%S").unwrap();
    let filter = Filter {
        since_date: Some(date),
        ..Default::default()
    };
    let filtered = read_events_filtered("messages.html", filter);
    assert_events!(&filtered,
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 11:05:13\")",
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 17:02:54\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:03:04\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:05:13\")",
        "BodyExtracted(\"W-what do you think? I hope you like it (´･ω･`)\")"
    );
}

#[test]
fn it_filters_by_short_name_blacklist() {
    let mut blacklist = std::collections::BTreeSet::new();
    blacklist.insert("sota");
    let filter = Filter {
        short_name_blacklist: Some(blacklist),
        ..Default::default()
    };
    let filtered = read_events_filtered("messages.html", filter);
    assert_events!(&filtered,
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 17:02:54\")",
        "BodyExtracted(\"🤔🤔🤔\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")"
    );
}

#[test]
fn it_filters_by_short_name_whitelist() {
    let mut whitelist = std::collections::BTreeSet::new();
    whitelist.insert("sota");
    let filter = Filter {
        short_name_whitelist: Some(whitelist),
        ..Default::default()
    };
    let filtered = read_events_filtered("messages.html", filter);
    assert_events!(&filtered,
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 11:05:13\")",
        "BodyExtracted(\"Hi Denko\\n\\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)\")",
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:03:04\")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:05:13\")",
        "BodyExtracted(\"W-what do you think? I hope you like it (´･ω･`)\")"
    );
}
