use joebot_message_parser::reader::{fold_html, MessageEvent, EventResult};

macro_rules! fixture {
    ($name: expr) => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join($name)
    };
}

#[derive(Default)]
struct Message {
    full_name: String,
    short_name: String,
    date: String,
    body: String,
}

fn msgid_body<P>(id: i32, path: P) -> Message
where P: AsRef<std::path::Path> {
    let mut msgid = -1;
    fold_html(
        fixture!(path),
        Default::default(),
        |mut msg: Message, event| {
            //println!("Event: {:?}", event);
            match event {
                MessageEvent::Start => {
                    msgid += 1;
                    EventResult::Consumed(msg)
                }
                MessageEvent::FullNameExtracted(full_name) if msgid == id => {
                    msg.full_name.push_str(full_name);
                    EventResult::Consumed(msg)
                }
                MessageEvent::ShortNameExtracted(short_name) if msgid == id => {
                    msg.short_name.push_str(short_name);
                    EventResult::Consumed(msg)
                }
                MessageEvent::DateExtracted(date) if msgid == id => {
                    msg.date.push_str(date);
                    EventResult::Consumed(msg)
                }
                MessageEvent::BodyExtracted(body) if msgid == id => {
                    msg.body = body;
                    EventResult::Consumed(msg)
                }
                _ => EventResult::Consumed(msg),
            }
        },
    )
    .unwrap()
}

#[test]
fn it_parses_messages_user_mentions() {
    let msg = msgid_body(0, fixture!("messages.html"));
    assert_eq!("Sota", msg.full_name);
    assert_eq!("sota", msg.short_name);
    assert_eq!("2018.01.21 11:05:13", msg.date);
    assert_eq!(
        "Hi Denko\n\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)",
        msg.body
    );
}

#[test]
fn it_parses_emoji() {
    let Message { body, .. } = msgid_body(1, fixture!("messages.html"));
    assert_eq!("🤔🤔🤔", body);
}

#[test]
fn it_parses_attachments_without_body() {
    let empty_body_msg = msgid_body(2, fixture!("messages.html"));
    assert_eq!("", empty_body_msg.body);
    let next_msg = msgid_body(3, fixture!("messages.html"));
    assert_eq!(
        "W-what do you think? I hope you like it (´･ω･`)",
        next_msg.body
    );
}
