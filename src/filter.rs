use crate::reader::MessageEvent;
use chrono::NaiveDateTime;

#[derive(Default)]
pub struct Filter {
    pub min_date: Option<NaiveDateTime>,
}

impl Filter {
    pub fn filter_event<'a>(&self, event: MessageEvent<'a>) -> Option<MessageEvent<'a>> {
        match event {
            MessageEvent::DateExtracted(date) => {
                if let Some(min_date) = self.min_date {
                    let msg_date =
                        NaiveDateTime::parse_from_str(date, "%Y.%m.%d %H:%M:%S").unwrap();
                    if msg_date >= min_date {
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
