use vkopt_message_parser::reader::{fold_html, EventResult};

macro_rules! assert_events {
    ($actual: expr, $($expected: expr),+) => {
        assert_eq!($actual, &[$($expected.to_owned(),)+])
    };
}

fn read_events(fixture: &str) -> Vec<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);
    fold_html(path, Vec::new(), |mut vec, event| {
        vec.push(format!("{:?}", event));
        EventResult::Consumed(vec)
    })
    .unwrap()
}

#[test]
fn it_parses_text_messages() {
    let events = read_events("messages.html");
    assert_events!(
        &events[..6],
        "Start",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 11:05:13\")",
        "BodyExtracted(\"Hi Denko\\n\\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)\")",
        "Start"
    );
}

#[test]
fn it_parses_emoji() {
    let events = read_events("messages.html");
    assert_events!(
        &events[5..11],
        "Start",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 17:02:54\")",
        "BodyExtracted(\"🤔🤔🤔\")",
        "Start"
    );
}

#[test]
fn it_parses_attachments_without_body() {
    let events = read_events("messages.html");
    assert_events!(
        &events[10..15],
        "Start",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:03:04\")",
        "Start"
    );
    assert_events!(
        &events[14..],
        "Start",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:05:13\")",
        "BodyExtracted(\"W-what do you think? I hope you like it (´･ω･`)\")"
    );
}
