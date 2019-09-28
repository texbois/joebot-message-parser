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
        let folded: String = fold_html(inputs[0], String::new(), |mut acc, event| match event {
            MessageEvent::BodyExtracted(body) if !body.is_empty() => {
                acc += &body;
                acc += "\n";
                EventResult::Consumed(acc)
            }
            _ => EventResult::Consumed(acc),
        })?;

        let mut out = std::fs::File::create(output)?;
        for acc in &[folded] {
            write!(&mut out, "{}", acc)?;
        }
        Ok(())
    }
}
