use vkopt_message_parser::filter::Filter;
use vkopt_message_parser::reader::{fold_html, EventResult, MessageEvent};
use std::collections::HashMap;

macro_rules! fixture {
    ($name: expr) => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join($name)
    };
}

#[test]
fn it_does_not_apply_filters_by_default() {
    let matched = matches(fixture!("messages.html"), Default::default());
    for id in [0, 1, 2, 3].into_iter() {
        assert!(matched[id].contains(&"FullNameExtracted"));
        assert!(matched[id].contains(&"ShortNameExtracted"));
        assert!(matched[id].contains(&"DateExtracted"));
        if *id != 2 {
            assert!(matched[id].contains(&"BodyExtracted"));
        }
    }
}

#[test]
fn it_filters_by_min_date() {
    let date =
        chrono::NaiveDateTime::parse_from_str("2018.01.22 00:00:00", "%Y.%m.%d %H:%M:%S").unwrap();
    let filter = Filter {
        since_date: Some(date),
        ..Default::default()
    };
    let matched = matches(fixture!("messages.html"), filter);
    for id in [0, 1].into_iter() {
        assert!(matched[id].contains(&"FullNameExtracted"));
        assert!(matched[id].contains(&"ShortNameExtracted"));
        assert_eq!(matched[id].len(), 2);
    }
    for id in [2, 3].into_iter() {
        assert!(matched[id].contains(&"FullNameExtracted"));
        assert!(matched[id].contains(&"ShortNameExtracted"));
        assert!(matched[id].contains(&"DateExtracted"));
        if *id != 2 {
            assert!(matched[id].contains(&"BodyExtracted"));
        }
    }
}

#[test]
fn it_filters_by_short_name_blacklist() {
    let mut blacklist = std::collections::BTreeSet::new();
    blacklist.insert("sota");
    let filter = Filter {
        short_name_blacklist: Some(blacklist),
        ..Default::default()
    };
    let matched = matches(fixture!("messages.html"), filter);
    for id in [0, 2, 3].into_iter() {
        assert!(matched[id].contains(&"FullNameExtracted"));
        assert_eq!(matched[id].len(), 1);
    }
    assert!(matched[&1].contains(&"FullNameExtracted"));
    assert!(matched[&1].contains(&"ShortNameExtracted"));
    assert!(matched[&1].contains(&"DateExtracted"));
    assert!(matched[&1].contains(&"BodyExtracted"));
}

#[test]
fn it_filters_by_short_name_whitelist() {
    let mut whitelist = std::collections::BTreeSet::new();
    whitelist.insert("sota");
    let filter = Filter {
        short_name_whitelist: Some(whitelist),
        ..Default::default()
    };
    let matched = matches(fixture!("messages.html"), filter);
    for id in [0, 2, 3].into_iter() {
        assert!(matched[id].contains(&"FullNameExtracted"));
        assert!(matched[id].contains(&"ShortNameExtracted"));
        assert!(matched[id].contains(&"DateExtracted"));
        if *id != 2 {
            assert!(matched[id].contains(&"BodyExtracted"));
            assert_eq!(matched[id].len(), 4);
        }
        else {
            assert_eq!(matched[id].len(), 3);
        }
    }
    assert!(matched[&1].contains(&"FullNameExtracted"));
    assert_eq!(matched[&1].len(), 1);
}

fn matches<P>(path: P, filter: Filter) -> HashMap<i32, Vec<&'static str>>
where P: AsRef<std::path::Path> {
    let mut msgid = -1;
    fold_html(
        fixture!(path),
        HashMap::new(),
        |mut matches: HashMap<i32, Vec<&'static str>>, event| {
            if let MessageEvent::Start(..) = event {
                msgid += 1;
                EventResult::Consumed(matches)
            }
            else {
                match filter.filter_event(event) {
                    Some(event) => {
                        match event {
                            MessageEvent::FullNameExtracted(_) => {
                                matches
                                    .entry(msgid)
                                    .or_insert(Vec::new())
                                    .push("FullNameExtracted");
                            }
                            MessageEvent::ShortNameExtracted(_) => {
                                matches
                                    .entry(msgid)
                                    .or_insert(Vec::new())
                                    .push("ShortNameExtracted");
                            }
                            MessageEvent::DateExtracted(_) => {
                                matches
                                    .entry(msgid)
                                    .or_insert(Vec::new())
                                    .push("DateExtracted");
                            }
                            MessageEvent::BodyExtracted(_) => {
                                matches
                                    .entry(msgid)
                                    .or_insert(Vec::new())
                                    .push("BodyExtracted");
                            }
                            _ => (),
                        };
                        EventResult::Consumed(matches)
                    }
                    _ => EventResult::SkipMessage(matches),
                }
            }
        },
    )
    .unwrap()
}
