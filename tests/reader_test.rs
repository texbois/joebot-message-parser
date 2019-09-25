use joebot_message_parser::reader::fold_html;

macro_rules! fixture {
    ($name: expr) => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join($name)
    };
}

#[test]
fn it_parses_user_mentions() {
    let body = fold_html(fixture!("messages.html"), String::new(), |acc, m| {
        // Grab the first message
        if acc.is_empty() {
            m.body
        }
        else {
            acc
        }
    })
    .unwrap();
    assert_eq!(
        "Hi Denko\n\nI’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)",
        body
    );
}
