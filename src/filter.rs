use crate::reader::MessageEvent;
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Filter {
    pub min_date: Option<NaiveDateTime>,
    pub short_name_blacklist: Option<BTreeSet<String>>,
}

impl Filter {
    pub fn filter_event<'a>(&self, event: MessageEvent<'a>) -> Option<MessageEvent<'a>> {
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
                if let Some(ref min_date) = self.min_date {
                    let msg_date =
                        NaiveDateTime::parse_from_str(date, "%Y.%m.%d %H:%M:%S").unwrap();
                    if msg_date >= *min_date {
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
