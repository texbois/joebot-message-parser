use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{local_name, parse_document, ParseOpts};
use regex::Regex;
use std::path::Path;

lazy_static! {
    static ref USER_MENTION_RE: Regex = Regex::new(r"\[id\d+\|(?P<name>[^\]]+)\]").unwrap();
}

pub struct Message {
    pub body: String,
}

pub fn fold_html<P, A, F>(path: P, init: A, mut reducer: F) -> std::io::Result<A>
where
    P: AsRef<Path>,
    F: FnMut(A, Message) -> A,
{
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .from_file(path)?;
    let acc = fold_messages(dom.document, init, &mut reducer);
    Ok(acc)
}

fn fold_messages<A, F>(node: Handle, init: A, reducer: &mut F) -> A
where
    F: FnMut(A, Message) -> A,
{
    if let NodeData::Element {
        ref name,
        ref attrs,
        ..
    } = node.data
    {
        if name.local == local_name!("div") && class_attr_eq(&attrs.borrow(), "msg_item") {
            let message = parse_message(node);
            return reducer(init, message);
        }
        if name.local == local_name!("head") {
            return init;
        }
    }

    let mut acc = init;
    for child in node.children.borrow().iter() {
        acc = fold_messages(child.clone(), acc, reducer);
    }

    acc
}

fn parse_message(node: Handle) -> Message {
    let mut body = String::new();

    for child in node.children.borrow().iter() {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            if name.local != local_name!("div") {
                continue;
            }
            if class_attr_eq(&attrs.borrow(), "from") {
                let inner = child.children.borrow();
                assert!(
                    inner.len() == 6,
                    "expected .from to have 6 child nodes, got {}",
                    inner.len()
                );

                let full_name =
                    if let NodeData::Text { ref contents } = inner[1].children.borrow()[0].data {
                        contents.borrow().to_string()
                    } else {
                        panic!("Expected the 2nd .from child to contain a text node");
                    };

                let screen_name =
                    if let NodeData::Text { ref contents } = inner[3].children.borrow()[0].data {
                        contents.borrow()[1..].to_string()
                    } else {
                        panic!("Expected the 4th .from child to contain a text node")
                    };
            } else if class_attr_eq(&attrs.borrow(), "msg_body") {
                for body_child in child.children.borrow().iter() {
                    match body_child.data {
                        NodeData::Text { ref contents } => {
                            body += &contents.borrow();
                        }
                        NodeData::Element {
                            ref name,
                            ref attrs,
                            ..
                        } => {
                            if name.local == local_name!("div")
                                && class_attr_eq(&attrs.borrow(), "emoji")
                            {
                                body += &attr_value(&attrs.borrow(), local_name!("alt")).unwrap();
                            } else if name.local == local_name!("br") {
                                body += "\n";
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    body = USER_MENTION_RE.replace_all(&body, "$name").to_string();

    Message { body }
}

fn class_attr_eq(attrs: &Vec<html5ever::Attribute>, value: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.name.local == local_name!("class") && *a.value == *value)
}

fn attr_value(
    attrs: &Vec<html5ever::Attribute>,
    name: html5ever::LocalName,
) -> Option<&tendril::StrTendril> {
    attrs
        .iter()
        .find(|a| a.name.local == name)
        .map(|a| &a.value)
}
