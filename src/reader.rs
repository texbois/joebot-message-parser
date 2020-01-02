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

macro_rules! raise_event_and_advance_state {
    ($reducer: expr, $acc: ident, $state: ident, $event: expr, $next_state: expr) => {
        match $reducer($acc, $event) {
            EventResult::Consumed(next_acc) => {
                $acc = next_acc;
                $state = $next_state;
            }
            EventResult::SkipMessage(next_acc) => {
                $acc = next_acc;
                $state = ParseState::NoMessage;
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
    let mut state = ParseState::Prelude;
    let mut acc = init;

    let mut msg_level = 0;
    let mut attachment_div_level = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match state {
                    // There's an <hr> tag right before the first msg_item
                    ParseState::Prelude if e.name() == b"hr" => state = ParseState::NoMessage,
                    ParseState::NoMessage | ParseState::MessageBodyExtracted
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_item") =>
                    {
                        raise_event_and_advance_state!(
                            reducer,
                            acc,
                            state,
                            MessageEvent::Start(msg_level),
                            ParseState::MessageStart
                        );
                    }
                    ParseState::MessageStart if e.name() == b"b" => {
                        state = ParseState::MessageFullNameStart
                    }
                    ParseState::MessageFullNameExtracted if e.name() == b"a" => {
                        state = ParseState::MessageShortNameStart
                    }
                    ParseState::MessageDateExtracted
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_body") =>
                    {
                        state = ParseState::MessageBodyStart(String::new())
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
                        state = ParseState::MessageAttachmentsStart
                    }
                    ParseState::MessageDateExtracted
                    | ParseState::MessageBodyExtracted
                    | ParseState::NoMessage
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"fwd") =>
                    {
                        msg_level += 1;
                        attachment_div_level += 2; // div class="att_head" + div class="fwd"
                        state = ParseState::NoMessage;
                    }
                    _ => {}
                };
            }
            Ok(Event::Text(e)) => match state {
                ParseState::MessageFullNameStart => {
                    let full_name = reader.decode(e.escaped())?;
                    raise_event_and_advance_state!(
                        reducer,
                        acc,
                        state,
                        MessageEvent::FullNameExtracted(full_name),
                        ParseState::MessageFullNameExtracted
                    );
                }
                ParseState::MessageShortNameStart => {
                    let short_name = reader.decode(e.escaped())?;
                    raise_event_and_advance_state!(
                        reducer,
                        acc,
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
                            reducer,
                            acc,
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
            Ok(Event::Empty(ref e)) => match state {
                ParseState::MessageBodyStart(ref mut body) if e.name() == b"br" => {
                    body.push_str("\n")
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match state {
                ParseState::MessageShortNameExtracted => state = ParseState::MessageDateStart,
                ParseState::MessageBodyStart(body) if e.name() == b"div" => {
                    attachment_div_level += 1; // msg_body's closing tag
                    raise_event_and_advance_state!(
                        reducer,
                        acc,
                        state,
                        MessageEvent::BodyExtracted(body),
                        ParseState::MessageBodyExtracted
                    );
                }
                ParseState::MessageAttachmentsStart if e.name() == b"div" => {
                    state = ParseState::NoMessage;
                }
                ParseState::MessageBodyExtracted
                | ParseState::NoMessage
                    if e.name() == b"div" =>
                {
                    if attachment_div_level > 0 {
                        attachment_div_level -= 1;
                        if attachment_div_level == 0 && msg_level > 0 {
                            msg_level -= 1;
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
    Ok(acc)
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
