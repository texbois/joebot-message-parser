use crate::filter::Filter;

mod text;

pub use text::TextWriter;

pub trait Writer {
    fn write<'a>(
        inputs: Vec<&'a str>,
        output: &'a str,
        filter: &Filter<'a>,
    ) -> quick_xml::Result<()>;
}
