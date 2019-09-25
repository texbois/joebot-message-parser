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
    Start,
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
    ($reducer: expr, $acc: ident, $state:ident, $event: expr, $next_state: expr) => {
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
                            MessageEvent::Start,
                            ParseState::MessageStart
                        );
                    }
                    ParseState::MessageStart if e.name() == b"b" => {
                        state = ParseState::MessageFullNameStart
                    }
                    ParseState::MessageFullNameExtracted if e.name() == b"a" => {
                        state = ParseState::MessageShortNameStart
                    }
                    ParseState::MessageShortNameExtracted if e.name() == b"a" => {
                        state = ParseState::MessageDateStart
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
                    let date = reader.decode(e.escaped())?;
                    raise_event_and_advance_state!(
                        reducer,
                        acc,
                        state,
                        MessageEvent::DateExtracted(date),
                        ParseState::MessageDateExtracted
                    );
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
                ParseState::MessageBodyStart(body) if e.name() == b"div" => {
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
                _ => (),
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
