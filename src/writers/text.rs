use crate::filter::Filter;
use crate::reader::{fold_html, EventResult, MessageEvent};
use std::io::Write;

pub struct TextWriter<'a> {
    pub delimiter: &'a str,
}

impl<'a> TextWriter<'a> {
    pub fn write<'w>(
        &self,
        inputs: Vec<&'w str>,
        output: &'w str,
        filter: &Filter<'w>,
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
                                acc += self.delimiter;
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
}
