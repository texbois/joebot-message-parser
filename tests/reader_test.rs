use joebot_message_parser::reader::fold_html;

macro_rules! fixture {
    ($name: expr) => {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join($name)
    };
}

#[test]
fn it_cleans_up_user_mentions() {
    let body = fold_html(fixture!("messages.html"), String::new(), |_, m| m.body).unwrap();
    assert_eq!("Hi Denko, I’m drinking jasmine tea right now, thinking about what to have for dinner (´･ω･`)", body);
}
