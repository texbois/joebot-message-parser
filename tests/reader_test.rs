use joebot_message_parser::reader::fold_html;

macro_rules! fixture {
    ($name: expr) => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join($name)
    };
}

fn msgid_body<P>(id: i32, path: P) -> String
where P: AsRef<std::path::Path> {
    let mut msgid = -1;
    fold_html(fixture!(path), String::new(), |acc, m| {
        msgid += 1;
        if msgid == id {
            m.body
        }
        else {
            acc
        }
    })
    .unwrap()
}

#[test]
fn it_parses_messages_user_mentions() {
    let body = msgid_body(0, fixture!("messages.html"));
    assert_eq!(
        "Hi Denko\n\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)",
        body
    );
}

#[test]
fn it_parses_emoji() {
    let body = msgid_body(1, fixture!("messages.html"));
    assert_eq!("🤔🤔🤔", body);
}

#[test]
fn it_parses_attachments_without_body() {
    let empty_body = msgid_body(2, fixture!("messages.html"));
    assert_eq!("", empty_body);
    let next_message = msgid_body(3, fixture!("messages.html"));
    assert_eq!("W-what do you think? I hope you like it (´･ω･`)", next_message);
}
