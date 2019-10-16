use crate::reader::MessageEvent;
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Filter<'a> {
    pub since_date: Option<NaiveDateTime>,
    pub short_name_whitelist: Option<BTreeSet<&'a str>>,
    pub short_name_blacklist: Option<BTreeSet<&'a str>>,
}

impl<'a> Filter<'a> {
    pub fn filter_event<'e>(&self, event: MessageEvent<'e>) -> Option<MessageEvent<'e>> {
        match event {
            MessageEvent::ShortNameExtracted(name) if short_name_passes(self, name) => Some(event),
            MessageEvent::ShortNameExtracted(_) => None,
            MessageEvent::DateExtracted(date) if date_passes(self, date) => Some(event),
            MessageEvent::DateExtracted(_) => None,
            _ => Some(event),
        }
    }
}

fn short_name_passes<'a>(filter: &Filter<'a>, short_name: &'a str) -> bool {
    if let Some(ref whitelist) = filter.short_name_whitelist {
        whitelist.contains(short_name)
    }
    else if let Some(ref blacklist) = filter.short_name_blacklist {
        !blacklist.contains(short_name)
    }
    else {
        true
    }
}

fn date_passes<'a>(filter: &Filter<'a>, date: &'a str) -> bool {
    if let Some(ref since_date) = filter.since_date {
        let msg_date = NaiveDateTime::parse_from_str(date, "%Y.%m.%d %H:%M:%S").unwrap();
        msg_date >= *since_date
    }
    else {
        true
    }
}
