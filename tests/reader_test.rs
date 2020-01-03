use vkopt_message_parser::reader::MessageEvent;
mod test_helper;
use test_helper::*;

#[test]
fn it_parses_text_messages() {
    let events = read_events("messages.html");
    assert_events!(
        &events[..6],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 11:05:13\")",
        "BodyExtracted(\"Hi Denko\\n\\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)\")",
        "Start(0)"
    );
}

#[test]
fn it_parses_emoji() {
    let events = read_events("messages.html");
    assert_events!(
        &events[5..11],
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 17:02:54\")",
        "BodyExtracted(\"🤔🤔🤔\")",
        "Start(0)"
    );
}

#[test]
fn it_parses_attachments_without_body() {
    let events = read_events("messages.html");
    assert_events!(
        &events[10..15],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:03:04\")",
        "Start(0)"
    );
    assert_events!(
        &events[14..],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:05:13\")",
        "BodyExtracted(\"W-what do you think? I hope you like it (´･ω･`)\")"
    );
}

#[test]
fn it_parses_forwarded_messages_with_arbitrary_nesting() {
    let events = read_events("messages_forwarded.html");
    assert_events!(
        &events,
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:18\")",
        "BodyExtracted(\"take it and leave\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:02:58\")",
        "BodyExtracted(\"pwetty pwease\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:03:04\")",
        "BodyExtracted(\"pwease don\\\'t ignore me (´･ω･`)\")",
        "Start(2)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 20:48:19\")",
        "BodyExtracted(\"how about now? (´･ω･`)\")",
        "Start(3)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 20:48:07\")",
        "BodyExtracted(\"ugh you just won\\\'t leave me alone will you\")",
        "Start(3)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 20:48:10\")",
        "BodyExtracted(\"I\\\'ll do it\")",
        "Start(1)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:06\")",
        "BodyExtracted(\"tomorrow maybe\")"
    );
}

#[test]
fn it_skips_forwarded_messages() {
    let events = read_events_skipping("messages_forwarded.html", |e| match e {
        MessageEvent::DateExtracted("2018.01.21 20:48:19") => false,
        _ => true,
    });
    assert_events!(
        &events,
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:18\")",
        "BodyExtracted(\"take it and leave\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:02:58\")",
        "BodyExtracted(\"pwetty pwease\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:03:04\")",
        "BodyExtracted(\"pwease don\\\'t ignore me (´･ω･`)\")",
        "Start(2)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 20:48:19\")",
        "Start(1)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:06\")",
        "BodyExtracted(\"tomorrow maybe\")"
    );
}

#[test]
fn it_skips_forwarded_messages_2() {
    let events = read_events_skipping("messages_forwarded.html", |e| match e {
        MessageEvent::Start(1) => false,
        _ => true,
    });
    assert_events!(
        &events,
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:18\")",
        "BodyExtracted(\"take it and leave\")",
        "Start(1)",
        "Start(1)",
        "Start(1)"
    );
}
