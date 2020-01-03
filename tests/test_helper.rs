use vkopt_message_parser::reader::{fold_html, EventResult, MessageEvent};

#[macro_export]
macro_rules! assert_events {
    ($actual: expr, $($expected: expr),+) => {
        assert_eq!($actual.to_vec(), vec![$($expected.to_owned(),)+])
    };
}

pub fn read_events(fixture: &str) -> Vec<String> {
    read_events_skipping(fixture, |_| true)
}

pub fn read_events_skipping<P: Fn(MessageEvent) -> bool>(fixture: &str, pred: P) -> Vec<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);
    fold_html(path, Vec::new(), |mut vec, event| {
        vec.push(format!("{:?}", event));
        if pred(event) {
            EventResult::Consumed(vec)
        } else {
            EventResult::SkipMessage(vec)
        }
    })
    .unwrap()
}
