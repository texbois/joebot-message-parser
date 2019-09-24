use joebot_message_parser::reader;

fn main() {
    let text = reader::fold_html("messages.html", String::new(), |mut acc, msg: reader::Message| {
        if !msg.body.is_empty() {
            acc += &msg.body;
            acc += "\n";
        }
        acc
    }).unwrap();
    std::fs::write("text", text).unwrap();
}
