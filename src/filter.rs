use crate::reader::MessageEvent;
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Filter<'a> {
    pub since_date: Option<NaiveDateTime>,
    pub short_name_blacklist: Option<BTreeSet<&'a str>>,
}

impl<'a> Filter<'a> {
    pub fn filter_event<'e>(&self, event: MessageEvent<'e>) -> Option<MessageEvent<'e>> {
        match event {
            MessageEvent::ShortNameExtracted(short_name) => {
                if let Some(ref blacklist) = self.short_name_blacklist {
                    if blacklist.contains(short_name) {
                        None
                    }
                    else {
                        Some(event)
                    }
                }
                else {
                    Some(event)
                }
            }
            MessageEvent::DateExtracted(date) => {
                if let Some(ref since_date) = self.since_date {
                    let msg_date =
                        NaiveDateTime::parse_from_str(date, "%Y.%m.%d %H:%M:%S").unwrap();
                    if msg_date >= *since_date {
                        Some(event)
                    }
                    else {
                        None
                    }
                }
                else {
                    Some(event)
                }
            }
            _ => Some(event),
        }
    }
}
