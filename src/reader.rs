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
    WallPartExtracted(&'a str),
    RawAttachmentPartExtracted(&'a str),
    AttachmentExtracted {
        kind: MessageAttachmentKind,
        url: &'a str,
        vk_obj: &'a str,
        description: &'a str,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MessageAttachmentKind {
    Doc,
    Photo,
    Video,
    Audio,
    Sticker,
    Location,
    Wall,
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
    MessageAttachmentsPrelude,
    MessageAttachmentStart,
    MessageAttachmentHeadStart(MessageAttachmentKind),
    MessageAttachmentBodyStart(MessageAttachmentKind, String),
    MessageAttachmentWallBodyStart,
    MessageAttachmentRawBodyStart,
    MessageAttachmentEpilogue,
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
            Some(max_level) if $state.msg_level > max_level => {}
            Some(_) if $state.at != MessageStart => {}
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

macro_rules! q {
    ($event: ident, $tag: literal, $attr: literal) => {
        $event.name() == $tag && $event.attributes_raw().contains_substring($attr)
    };
    ($event: ident, $tag: literal) => {
        $event.name() == $tag
    };
}

fn fold_with_reader<B, A, F>(mut reader: Reader<B>, init: A, reducer: F) -> quick_xml::Result<A>
where
    B: std::io::BufRead,
    F: for<'e> FnMut(A, MessageEvent<'e>) -> EventResult<A>,
{
    use MessageEvent::*;
    use ParseState::*;

    let mut buf = Vec::new();
    let mut state = ParseStateHolder {
        at: Prelude,
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
                Prelude if q!(e, b"hr") => state.advance(NoMessage),
                NoMessage | MessageBodyExtracted if q!(e, b"div", b"\"msg_item\"") => {
                    state.advance(MessageStart);
                    msg_event!(state, Start(state.msg_level));
                }
                MessageStart if q!(e, b"b") => {
                    state.advance(MessageFullNameStart);
                }
                MessageFullNameExtracted if q!(e, b"a") => {
                    state.advance(MessageShortNameStart);
                }
                MessageDateExtracted if q!(e, b"div", b"\"msg_body\"") => {
                    state.advance(MessageBodyStart);
                }
                MessageBodyStart if q!(e, b"img", b"\"emoji\"") => {
                    if let Some(alt) = get_attr(&mut e.attributes(), b"alt") {
                        msg_event!(state, BodyPartExtracted(reader.decode(&alt)?));
                    }
                }
                MessageDateExtracted if q!(e, b"div") && e.attributes_raw().is_empty() => {
                    state.advance(MessageChatActionStart);
                }
                MessageDateExtracted | MessageBodyExtracted if q!(e, b"div", b"\"attacments\"") => {
                    state.advance(MessageAttachmentsPrelude)
                }
                MessageAttachmentsPrelude | MessageBodyExtracted
                    if q!(e, b"div", b"\"attacment\"") =>
                {
                    state.advance(MessageAttachmentStart)
                }
                MessageAttachmentsPrelude | MessageBodyExtracted
                    if q!(e, b"div", b"\"att_head\"") =>
                {
                    state.advance(MessageForwardedStart)
                }
                MessageAttachmentStart if q!(e, b"div", b"att_ico") => {
                    // Matching the last four symbols of the class only -- why? Just for lulz
                    let attrs = e.attributes_raw();
                    let kind = match &attrs[attrs.len() - 5..attrs.len() - 1] {
                        b"_doc" => MessageAttachmentKind::Doc,
                        b"udio" => MessageAttachmentKind::Audio,
                        b"ideo" => MessageAttachmentKind::Video,
                        b"hoto" => MessageAttachmentKind::Photo,
                        b"cker" => MessageAttachmentKind::Sticker,
                        b"_geo" => MessageAttachmentKind::Location,
                        b"wall" => MessageAttachmentKind::Wall,
                        _ => panic!("Unsupported attachment container: {:?}", e),
                    };
                    state.advance(MessageAttachmentHeadStart(kind));
                }
                MessageAttachmentStart if q!(e, b"pre") => {
                    state.advance(MessageAttachmentRawBodyStart);
                }
                MessageAttachmentHeadStart(kind) if q!(e, b"a") => {
                    let mut attrs = e.attributes();
                    let href = get_attr(&mut attrs, b"href").unwrap_or(Cow::Borrowed(&[]));
                    let src = reader.decode(&href)?.to_owned();
                    state.advance(MessageAttachmentBodyStart(kind, src));
                }
                MessageAttachmentEpilogue if q!(e, b"div", b"\"att_wall_text\"") => {
                    state.advance(MessageAttachmentWallBodyStart);
                }
                MessageForwardedStart if q!(e, b"div", b"\"fwd\"") => {
                    state.msg_level += 1;
                    state.fwd_closed = false;
                    state.advance(NoMessage);
                }
                _ => {}
            },
            Ok(Event::Text(e)) => match state.at {
                MessageFullNameStart => {
                    state.advance(MessageFullNameExtracted);
                    msg_event!(state, FullNameExtracted(reader.decode(e.escaped())?));
                }
                MessageShortNameStart => {
                    state.advance(MessageShortNameExtracted);
                    msg_event!(
                        state,
                        ShortNameExtracted(&reader.decode(e.escaped())?[1..]) // skip the leading @
                    );
                }
                MessageDateStart => {
                    let maybe_date = e.escaped().trim();
                    if !maybe_date.is_empty() {
                        state.advance(MessageDateExtracted);
                        msg_event!(state, DateExtracted(reader.decode(maybe_date)?));
                    }
                }
                MessageBodyStart => {
                    let unescaped = &e.unescaped().unwrap_or(Cow::from(e.escaped()));
                    let text = reader.decode(&unescaped)?;
                    if text.contains('[') {
                        let re_text = USER_MENTION_RE.replace_all(text, "$name");
                        msg_event!(state, BodyPartExtracted(&re_text));
                    } else if !text.is_empty() {
                        msg_event!(state, BodyPartExtracted(&text));
                    }
                }
                MessageAttachmentBodyStart(kind, ref url) => {
                    let unescaped = &e.unescaped().unwrap_or(Cow::from(e.escaped()));
                    let info = reader.decode(&unescaped)?.trim();
                    let (vk_obj, description) = if info.starts_with('[') {
                        let mut info_split = info[1..].splitn(2, ']');
                        let vk_obj = info_split.next().unwrap_or("");
                        let description = info_split.next().unwrap_or("").trim();
                        (vk_obj, description)
                    } else {
                        ("", info)
                    };
                    msg_event!(
                        state,
                        AttachmentExtracted {
                            kind,
                            url,
                            vk_obj,
                            description
                        }
                    );
                    state.advance(MessageAttachmentEpilogue);
                }
                MessageAttachmentRawBodyStart => {
                    let unescaped = &e.unescaped().unwrap_or(Cow::from(e.escaped()));
                    let data = reader.decode(&unescaped)?;
                    msg_event!(state, RawAttachmentPartExtracted(&data));
                }
                MessageAttachmentWallBodyStart => {
                    let unescaped = &e.unescaped().unwrap_or(Cow::from(e.escaped()));
                    let text = reader.decode(&unescaped)?;
                    msg_event!(state, WallPartExtracted(&text));
                }
                _ => (),
            },
            Ok(Event::Empty(ref e)) => match state.at {
                MessageBodyStart if q!(e, b"br") => {
                    msg_event!(state, BodyPartExtracted("\n"));
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match state.at {
                MessageShortNameExtracted => state.advance(MessageDateStart),
                MessageBodyStart
                | MessageAttachmentWallBodyStart
                | MessageChatActionStart
                | MessageAttachmentEpilogue
                    if q!(e, b"div") =>
                {
                    state.advance(MessageBodyExtracted);
                }
                MessageAttachmentRawBodyStart if q!(e, b"pre") => {
                    state.advance(MessageBodyExtracted)
                }
                MessageBodyExtracted if q!(e, b"div") => {
                    state.advance(NoMessage);
                }
                NoMessage if q!(e, b"div") => {
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
