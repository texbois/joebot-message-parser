use vkopt_message_parser::reader::MessageEvent;
mod test_helper;
use test_helper::*;

#[test]
fn it_skips_chat_actions() {
    let events = read_events("messages.html");
    assert_events!(
        &events[..5],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 13:53:59\")",
        "Start(0)"
    );
}

#[test]
fn it_parses_text_messages() {
    let events = read_events("messages.html");
    assert_events!(
        &events[4..13],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 11:05:13\")",
        "BodyPartExtracted(\"Hi Denko\")",
        "BodyPartExtracted(\"\\n\")",
        "BodyPartExtracted(\"\\n\")",
        "BodyPartExtracted(\"I’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)\")",
        "Start(0)"
    );
}

#[test]
fn it_parses_emoji() {
    let events = read_events("messages.html");
    assert_events!(
        &events[12..20],
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 17:02:54\")",
        "BodyPartExtracted(\"🤔\")",
        "BodyPartExtracted(\"🤔\")",
        "BodyPartExtracted(\"🤔\")",
        "Start(0)"
    );
}

#[test]
fn it_parses_attachments_without_body() {
    let events = read_events("messages.html");
    assert_events!(
        &events[19..24],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:03:04\")",
        "Start(0)"
    );
    assert_events!(
        &events[23..],
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.22 10:05:13\")",
        "BodyPartExtracted(\"W-what do you think? I hope you like it (´･ω･`)\")"
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
        "BodyPartExtracted(\"take it and leave\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:02:58\")",
        "BodyPartExtracted(\"pwetty pwease\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:03:04\")",
        "BodyPartExtracted(\"pwease don\\\'t ignore me (´･ω･`)\")",
        "Start(2)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 20:48:19\")",
        "BodyPartExtracted(\"how about now? (´･ω･`)\")",
        "Start(3)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 20:48:07\")",
        "BodyPartExtracted(\"ugh you just won\\\'t leave me alone will you\")",
        "Start(3)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 20:48:10\")",
        "BodyPartExtracted(\"I\\\'ll do it\")",
        "Start(1)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:06\")",
        "BodyPartExtracted(\"tomorrow maybe\")"
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
        "BodyPartExtracted(\"take it and leave\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:02:58\")",
        "BodyPartExtracted(\"pwetty pwease\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2019.01.02 07:03:04\")",
        "BodyPartExtracted(\"pwease don\\\'t ignore me (´･ω･`)\")",
        "Start(2)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 20:48:19\")",
        "Start(1)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2019.01.02 07:03:06\")",
        "BodyPartExtracted(\"tomorrow maybe\")"
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
        "BodyPartExtracted(\"take it and leave\")",
        "Start(1)",
        "Start(1)",
        "Start(1)"
    );
}

#[test]
fn it_parses_forwarded_messages_with_attachments() {
    let events = read_events("messages_forwarded_att.html");

    assert_events!(
        &events,
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 19:00:55\")",
        "Start(0)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 19:02:09\")",
        "BodyPartExtracted(\"I hope this time is the last time for real\")",
        "Start(1)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 18:59:35\")",
        "BodyPartExtracted(\"thankuwu:3:3:3:3:3\")",
        "Start(2)",
        "FullNameExtracted(\"Denko\")",
        "ShortNameExtracted(\"denko\")",
        "DateExtracted(\"2018.01.21 18:58:09\")",
        "BodyPartExtracted(\" \")",
        "Start(0)",
        "FullNameExtracted(\"Sota\")",
        "ShortNameExtracted(\"sota\")",
        "DateExtracted(\"2018.01.21 19:36:18\")",
        "BodyPartExtracted(\"don\\\'t be a meanie uwu you awe so bwutiful\")"
    );
}
