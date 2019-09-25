use quick_xml::events::{attributes::Attributes, Event};
use quick_xml::Reader;
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;

lazy_static! {
    static ref USER_MENTION_RE: Regex = Regex::new(r"\[id\d+\|(?P<name>[^\]]+)\]").unwrap();
}

pub struct Message {
    pub body: String,
    pub full_name: String,
    pub short_name: String,
    pub date: String,
}

pub fn fold_html<P, A, F>(path: P, init: A, mut reducer: F) -> quick_xml::Result<A>
where
    P: AsRef<Path>,
    F: FnMut(A, Message) -> A,
{
    let mut reader = Reader::from_file(path)?;
    reader.check_end_names(false);

    fold_with_reader(reader, init, |acc, mut message| {
        if !message.body.is_empty() {
            message.body = USER_MENTION_RE
                .replace_all(&message.body, "$name")
                .to_string();
        }
        reducer(acc, message)
    })
}

enum ParseState {
    Prelude,
    NoMessage,
    MessageStart,
    MessageFullNameStart,
    MessageFullNameExtracted {
        full_name: String,
    },
    MessageShortNameStart {
        full_name: String,
    },
    MessageShortNameExtracted {
        full_name: String,
        short_name: String,
    },
    MessageDateStart {
        full_name: String,
        short_name: String,
    },
    MessageDateExtracted(Message),
    MessageBody(Message),
    MessageBodyExtracted(Message),
    MessageAttachmentsHeader(Message),
    MessageAttachments {
        message: Message,
        div_nesting: u32,
    },
}

fn fold_with_reader<B, A, F>(mut reader: Reader<B>, init: A, mut reducer: F) -> quick_xml::Result<A>
where
    B: std::io::BufRead,
    F: FnMut(A, Message) -> A,
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
                    ParseState::NoMessage
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_item") =>
                    {
                        state = ParseState::MessageStart
                    }
                    ParseState::MessageStart if e.name() == b"b" => {
                        state = ParseState::MessageFullNameStart
                    }
                    ParseState::MessageFullNameExtracted { full_name } if e.name() == b"a" => {
                        state = ParseState::MessageShortNameStart { full_name }
                    }
                    ParseState::MessageShortNameExtracted {
                        full_name,
                        short_name,
                    } if e.name() == b"a" => {
                        state = ParseState::MessageDateStart {
                            full_name,
                            short_name,
                        }
                    }
                    ParseState::MessageDateExtracted(message)
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"msg_body") =>
                    {
                        state = ParseState::MessageBody(message)
                    }
                    ParseState::MessageBody(Message { ref mut body, .. })
                        if e.name() == b"img" && class_eq(&mut e.attributes(), b"emoji") =>
                    {
                        if let Some(alt) = get_attr(&mut e.attributes(), b"alt") {
                            body.push_str(reader.decode(&alt)?)
                        }
                    }
                    ParseState::MessageDateExtracted(message) |
                    ParseState::MessageBodyExtracted(message)
                        if e.name() == b"div" && class_eq(&mut e.attributes(), b"attacments") =>
                    {
                        state = ParseState::MessageAttachmentsHeader(message)
                    }
                    _ => {}
                };
            }
            Ok(Event::Text(e)) => match state {
                ParseState::MessageFullNameStart => {
                    let full_name = reader.decode(e.escaped())?.to_owned();
                    state = ParseState::MessageFullNameExtracted { full_name };
                }
                ParseState::MessageShortNameStart { full_name } => {
                    let short_name = reader.decode(e.escaped())?.to_owned();
                    state = ParseState::MessageShortNameExtracted {
                        full_name,
                        short_name,
                    };
                }
                ParseState::MessageDateStart {
                    full_name,
                    short_name,
                } => {
                    let date = reader.decode(e.escaped())?.to_owned();
                    state = ParseState::MessageDateExtracted(Message {
                        full_name,
                        short_name,
                        date,
                        body: String::new(),
                    });
                }
                ParseState::MessageBody(Message { ref mut body, .. }) => {
                    body.push_str(reader.decode(e.escaped())?);
                }
                _ => (),
            },
            Ok(Event::Empty(ref e)) => match state {
                ParseState::MessageBody(Message { ref mut body, .. }) if e.name() == b"br" => {
                    body.push_str("\n")
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match state {
                ParseState::MessageBody(message) if e.name() == b"div" => {
                    state = ParseState::MessageBodyExtracted(message)
                }
                ParseState::MessageAttachmentsHeader(message) if e.name() == b"div" => {
                    state = ParseState::MessageAttachments {
                        message,
                        div_nesting: 0,
                    }
                }
                ParseState::MessageBodyExtracted(message) |
                ParseState::MessageAttachments {
                    message,
                    div_nesting: 0,
                } if e.name() == b"div" => {
                    acc = reducer(acc, message);
                    state = ParseState::NoMessage
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
    attrs.any(|ar| match ar {
        Ok(a) => a.key == b"class" && a.value.as_ref() == cmp,
        _ => false,
    })
}

fn get_attr<'a>(attrs: &'a mut Attributes, key: &[u8]) -> Option<Cow<'a, [u8]>> {
    attrs.find_map(|ar| match ar {
        Ok(a) if a.key == key => Some(a.value),
        _ => None,
    })
}
