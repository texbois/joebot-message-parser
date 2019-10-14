use crate::filter::Filter;
use crate::reader::{fold_html, EventResult, MessageEvent};
use crate::writers::Writer;
use std::io::Write;

pub struct TextWriter;

impl Writer for TextWriter {
    fn write<'a>(
        inputs: Vec<&'a str>,
        output: &'a str,
        filter: &Filter<'a>,
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
                                acc += "\n";
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
