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
    Start(u32), // > 0 indicates the nesting level for forwarded messages
    FullNameExtracted(&'a str),
    ShortNameExtracted(&'a str),
    DateExtracted(&'a str),
    BodyPartExtracted(&'a str),
}

pub enum EventResult<A> {
    Consumed(A),
    SkipMessage(A),
}

pub fn fold_html<P, A, F>(path: P, init: A, reducer: F) -> quick_xml::Result<A>
where
    P: AsRef<Path>,
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    let mut reader = Reader::from_file(path)?;
    reader.check_end_names(false);

    fold_with_reader(reader, init, reducer)
}

#[derive(Debug, PartialEq)]
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
    MessageBodyStart,
    MessageBodyExtracted,
    MessageAttachments(u32),
    MessageAttachmentsExtracted,
    MessageForwardedStart,
    MessageChatActionStart,
}

struct ParseStateHolder<A, F>
where
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    at: ParseState,
    msg_level: u32,
    fwd_closed: bool,
    skip_level: Option<u32>,
    acc: A,
    reducer: F,
}

impl<A, F> ParseStateHolder<A, F>
where
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    fn advance(&mut self, new_state: ParseState) {
        self.at = new_state;
    }
}

macro_rules! msg_event {
    ($state: ident, $event: expr) => {
        match $state.skip_level {
            Some(max_level) if $state.msg_level > max_level => (),
            Some(_) if $state.at != ParseState::MessageStart => (),
            _ => match ($state.reducer)($state.acc, $event) {
                EventResult::Consumed(next_acc) => {
                    $state.acc = next_acc;
                    $state.skip_level = None;
                }
                EventResult::SkipMessage(next_acc) => {
                    $state.acc = next_acc;
                    $state.skip_level = Some($state.msg_level);
                }
            },
        }
    };
}

fn fold_with_reader<B, A, F>(mut reader: Reader<B>, init: A, reducer: F) -> quick_xml::Result<A>
where
    B: std::io::BufRead,
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    let mut buf = Vec::new();
    let mut state = ParseStateHolder {
        at: ParseState::Prelude,
        msg_level: 0,
        fwd_closed: false,
        skip_level: None,
        acc: init,
        reducer,
    };

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match state.at {
                // There's an <hr> tag right before the first msg_item
                ParseState::Prelude if e.name() == b"hr" => state.advance(ParseState::NoMessage),
                ParseState::NoMessage
                | ParseState::MessageBodyExtracted
                | ParseState::MessageAttachmentsExtracted
                    if e.name() == b"div" && e.attributes_raw().contains_substring(b"\"msg_item\"") =>
                {
                    state.advance(ParseState::MessageStart);
                    msg_event!(state, MessageEvent::Start(state.msg_level));
                }
                ParseState::MessageStart if e.name() == b"b" => {
                    state.advance(ParseState::MessageFullNameStart);
                }
                ParseState::MessageFullNameExtracted if e.name() == b"a" => {
                    state.advance(ParseState::MessageShortNameStart);
                }
                ParseState::MessageDateExtracted
                    if e.name() == b"div" && e.attributes_raw().contains_substring(b"\"msg_body\"") =>
                {
                    state.advance(ParseState::MessageBodyStart);
                }
                ParseState::MessageBodyStart
                    if e.name() == b"img" && e.attributes_raw().contains_substring(b"\"emoji\"") =>
                {
                    if let Some(alt) = get_attr(&mut e.attributes(), b"alt") {
                        msg_event!(state, MessageEvent::BodyPartExtracted(reader.decode(&alt)?));
                    }
                }
                ParseState::MessageDateExtracted
                    if e.name() == b"div" && e.attributes_raw().is_empty() =>
                {
                    state.advance(ParseState::MessageChatActionStart);
                }
                ParseState::MessageDateExtracted
                | ParseState::MessageBodyExtracted
                | ParseState::MessageAttachmentsExtracted
                    if e.name() == b"div" =>
                {
                    let mut attrs = e.attributes();
                    match get_attr(&mut attrs, b"class") {
                        Some(cls)
                            if cls.as_ref() == b"attacments" || cls.as_ref() == b"attacment" =>
                        {
                            state.advance(ParseState::MessageAttachments(0));
                        }
                        Some(cls) if cls.as_ref() == b"att_head" => {
                            state.advance(ParseState::MessageForwardedStart);
                        }
                        _ => (),
                    }
                }
                ParseState::MessageAttachments(nesting) if e.name() == b"div" => {
                    state.advance(ParseState::MessageAttachments(nesting + 1))
                }
                ParseState::MessageForwardedStart
                    if e.name() == b"div" && e.attributes_raw().contains_substring(b"\"fwd\"") =>
                {
                    state.msg_level += 1;
                    state.fwd_closed = false;
                    state.advance(ParseState::NoMessage);
                }
                _ => {}
            },
            Ok(Event::Text(e)) => match state.at {
                ParseState::MessageFullNameStart => {
                    state.advance(ParseState::MessageFullNameExtracted);
                    msg_event!(
                        state,
                        MessageEvent::FullNameExtracted(reader.decode(e.escaped())?)
                    );
                }
                ParseState::MessageShortNameStart => {
                    state.advance(ParseState::MessageShortNameExtracted);
                    msg_event!(
                        state,
                        MessageEvent::ShortNameExtracted(&reader.decode(e.escaped())?[1..]) // skip the leading @
                    );
                }
                ParseState::MessageDateStart => {
                    let maybe_date = e.escaped().trim();
                    if !maybe_date.is_empty() {
                        state.advance(ParseState::MessageDateExtracted);
                        msg_event!(
                            state,
                            MessageEvent::DateExtracted(reader.decode(maybe_date)?)
                        );
                    }
                }
                ParseState::MessageBodyStart => {
                    let text = reader.decode(e.escaped())?;
                    if text.contains('[') {
                        let re_text = USER_MENTION_RE.replace_all(text, "$name");
                        msg_event!(state, MessageEvent::BodyPartExtracted(&re_text));
                    } else if !text.is_empty() {
                        msg_event!(state, MessageEvent::BodyPartExtracted(&text));
                    }
                }
                _ => (),
            },
            Ok(Event::Empty(ref e)) => match state.at {
                ParseState::MessageBodyStart if e.name() == b"br" => {
                    msg_event!(state, MessageEvent::BodyPartExtracted("\n"));
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match state.at {
                ParseState::MessageShortNameExtracted => {
                    state.advance(ParseState::MessageDateStart)
                }
                ParseState::MessageBodyStart | ParseState::MessageChatActionStart
                    if e.name() == b"div" =>
                {
                    state.advance(ParseState::MessageBodyExtracted);
                }
                ParseState::MessageAttachments(nesting) if e.name() == b"div" => {
                    state.advance(if nesting > 0 {
                        ParseState::MessageAttachments(nesting - 1)
                    } else {
                        ParseState::MessageAttachmentsExtracted
                    });
                }
                ParseState::MessageAttachmentsExtracted | ParseState::MessageBodyExtracted
                    if e.name() == b"div" =>
                {
                    state.advance(ParseState::NoMessage);
                }
                ParseState::NoMessage if e.name() == b"div" => {
                    if state.msg_level > 0 {
                        if !state.fwd_closed {
                            state.fwd_closed = true;
                        } else {
                            state.msg_level -= 1;
                            state.fwd_closed = false;
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

fn get_attr<'a>(attrs: &'a mut Attributes, key: &[u8]) -> Option<Cow<'a, [u8]>> {
    attrs.with_checks(false).find_map(|ar| match ar {
        Ok(a) if a.key == key => Some(a.value),
        _ => None,
    })
}

// Based on https://stackoverflow.com/a/31102496/1726690
trait RawText {
    fn trim(&self) -> &Self;
    fn contains_substring(&self, sub: &[u8]) -> bool;
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

    fn contains_substring(&self, sub: &[u8]) -> bool {
        let mut s = self;
        while !s.is_empty() {
            if let Some(pos) = s.iter().position(|&c| c == sub[0]) {
                let endpos = pos + sub.len();
                if endpos > s.len() {
                    return false;
                }
                if &s[pos..pos + sub.len()] == sub {
                    return true;
                }
                s = &s[pos + 1..];
            }
        }
        false
    }
}
