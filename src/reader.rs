use quick_xml::events::{attributes::Attributes, Event};
use quick_xml::Reader;
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;

lazy_static! {
    static ref USER_MENTION_RE: Regex = Regex::new(r"\[id\d+\|(?P<name>[^\]]+)\]").unwrap();
}

#[derive(Debug)]
pub enum MessageEvent<'a> {
    Start(u32), // > 0 indicates the nesting level for forwarded message
    FullNameExtracted(&'a str),
    ShortNameExtracted(&'a str),
    DateExtracted(&'a str),
    BodyExtracted(String),
}

pub enum EventResult<A> {
    Consumed(A),
    SkipMessage(A),
}

pub fn fold_html<P, A, F>(path: P, init: A, mut reducer: F) -> quick_xml::Result<A>
where
    P: AsRef<Path>,
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    let mut reader = Reader::from_file(path)?;
    reader.check_end_names(false);

    fold_with_reader(reader, init, |acc, event| match event {
        MessageEvent::BodyExtracted(mut body) if !body.is_empty() => {
            if body.contains('[') {
                body = USER_MENTION_RE.replace_all(&body, "$name").into_owned();
            }
            reducer(acc, MessageEvent::BodyExtracted(body))
        }
        _ => reducer(acc, event),
    })
}

#[derive(Debug)]
enum ParseState {
    Prelude,
    NoMessage,
    MessageStart,
    MessageFullNameStart,
    MessageFullNameExtracted,
    MessageShortNameStart,
    MessageShortNameExtracted,
    MessageDateStart,
    MessageDateExtracted,
    MessageBodyStart(String),
    MessageBodyExtracted,
    MessageAttachmentsStart,
}

struct ParseStateHolder<A, F>
where
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    at: ParseState,
    msg_level: u32,
    attachment_div_level: u32,
    acc: A,
    reducer: F,
}

macro_rules! raise_event_and_advance_state {
    ($state: ident, $event: expr, $next_state: expr) => {
        match ($state.reducer)($state.acc, $event) {
            EventResult::Consumed(next_acc) => {
                $state.acc = next_acc;
                $state.at = $next_state;
            }
            EventResult::SkipMessage(next_acc) => {
                $state.acc = next_acc;
                $state.at = ParseState::NoMessage;
            }
        }
    };
}

fn fold_with_reader<B, A, F>(mut reader: Reader<B>, init: A, mut reducer: F) -> quick_xml::Result<A>
where
    B: std::io::BufRead,
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    let mut buf = Vec::new();
    let mut state = ParseStateHolder {
        at: ParseState::Prelude,
        msg_level: 0,
        attachment_div_level: 0,
        acc: init,
        reducer,
    };

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match state.at {
                // There's an <hr> tag right before the first msg_item
                ParseState::Prelude if e.name() == b"hr" => state.at = ParseState::NoMessage,
                ParseState::NoMessage | ParseState::MessageBodyExtracted
                    if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_item") =>
                {
                    raise_event_and_advance_state!(
                        state,
                        MessageEvent::Start(state.msg_level),
                        ParseState::MessageStart
                    );
                }
                ParseState::MessageStart if e.name() == b"b" => {
                    state.at = ParseState::MessageFullNameStart
                }
                ParseState::MessageFullNameExtracted if e.name() == b"a" => {
                    state.at = ParseState::MessageShortNameStart
                }
                ParseState::MessageDateExtracted
                    if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_body") =>
                {
                    state.at = ParseState::MessageBodyStart(String::new())
                }
                ParseState::MessageBodyStart(ref mut body)
                    if e.name() == b"img" && class_eq(&mut e.attributes(), b"emoji") =>
                {
                    if let Some(alt) = get_attr(&mut e.attributes(), b"alt") {
                        body.push_str(reader.decode(&alt)?)
                    }
                }
                ParseState::MessageDateExtracted | ParseState::MessageBodyExtracted
                    if e.name() == b"div" && class_eq(&mut e.attributes(), b"attacments") =>
                {
                    state.at = ParseState::MessageAttachmentsStart
                }
                ParseState::MessageDateExtracted
                | ParseState::MessageBodyExtracted
                | ParseState::NoMessage
                    if e.name() == b"div" && class_eq(&mut e.attributes(), b"fwd") =>
                {
                    state.msg_level += 1;
                    state.attachment_div_level += 2; // div class="att_head" + div class="fwd"
                    state.at = ParseState::NoMessage;
                }
                _ => {}
            },
            Ok(Event::Text(e)) => match state.at {
                ParseState::MessageFullNameStart => {
                    let full_name = reader.decode(e.escaped())?;
                    raise_event_and_advance_state!(
                        state,
                        MessageEvent::FullNameExtracted(full_name),
                        ParseState::MessageFullNameExtracted
                    );
                }
                ParseState::MessageShortNameStart => {
                    let short_name = reader.decode(e.escaped())?;
                    raise_event_and_advance_state!(
                        state,
                        MessageEvent::ShortNameExtracted(&short_name[1..]), // skip the leading @
                        ParseState::MessageShortNameExtracted
                    );
                }
                ParseState::MessageDateStart => {
                    let maybe_date = e.escaped().trim();
                    if !maybe_date.is_empty() {
                        let date = reader.decode(maybe_date)?;
                        raise_event_and_advance_state!(
                            state,
                            MessageEvent::DateExtracted(date),
                            ParseState::MessageDateExtracted
                        );
                    }
                }
                ParseState::MessageBodyStart(ref mut body) => {
                    body.push_str(reader.decode(e.escaped())?)
                }
                _ => (),
            },
            Ok(Event::Empty(ref e)) => match state.at {
                ParseState::MessageBodyStart(ref mut body) if e.name() == b"br" => {
                    body.push_str("\n")
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match state.at {
                ParseState::MessageShortNameExtracted => state.at = ParseState::MessageDateStart,
                ParseState::MessageBodyStart(body) if e.name() == b"div" => {
                    state.attachment_div_level += 1; // msg_body's closing tag
                    raise_event_and_advance_state!(
                        state,
                        MessageEvent::BodyExtracted(body),
                        ParseState::MessageBodyExtracted
                    );
                }
                ParseState::MessageAttachmentsStart if e.name() == b"div" => {
                    state.at = ParseState::NoMessage;
                }
                ParseState::MessageBodyExtracted | ParseState::NoMessage if e.name() == b"div" => {
                    if state.attachment_div_level > 0 {
                        state.attachment_div_level -= 1;
                        if state.attachment_div_level == 0 && state.msg_level > 0 {
                            state.msg_level -= 1;
                        }
                    }
                }
                _ => {}
            },
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    Ok(state.acc)
}

fn class_eq(attrs: &mut Attributes, cmp: &[u8]) -> bool {
    attrs.with_checks(false).any(|ar| match ar {
        Ok(a) => a.key == b"class" && a.value.as_ref() == cmp,
        _ => false,
    })
}

fn get_attr<'a>(attrs: &'a mut Attributes, key: &[u8]) -> Option<Cow<'a, [u8]>> {
    attrs.with_checks(false).find_map(|ar| match ar {
        Ok(a) if a.key == key => Some(a.value),
        _ => None,
    })
}

// Based on https://stackoverflow.com/a/31102496/1726690
trait RawText {
    fn trim(&self) -> &Self;
}

impl RawText for [u8] {
    fn trim(&self) -> &[u8] {
        fn is_not_whitespace(c: &u8) -> bool {
            *c != b' ' && *c != b'\r' && *c != b'\n'
        }

        if let Some(first) = self.iter().position(is_not_whitespace) {
            let last = self.iter().rposition(is_not_whitespace).unwrap();
            &self[first..last + 1]
        } else {
            &[]
        }
    }
}
