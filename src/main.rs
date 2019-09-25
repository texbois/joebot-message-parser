use joebot_message_parser::reader::{fold_html, MessageEvent};

fn main() {
    let text = fold_html(
        "messages.html",
        String::new(),
        |mut acc, event| match event {
            MessageEvent::BodyExtracted(body) if !body.is_empty() => {
                acc += &body;
                acc += "\n";
                acc
            },
            _ => acc
        }
    )
    .unwrap();
    std::fs::write("text", text).unwrap();
}
