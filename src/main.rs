use std::fs::File;

use html5ever::{ParseOpts, parse_document, local_name};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use regex::Regex;

fn main() {
    let mut messages_html = File::open("messages.html").unwrap();

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut messages_html)
        .unwrap();

    let text = retrieve_messages(dom.document);
    std::fs::write("text", text).unwrap();
}

fn retrieve_messages(node: Handle) -> String {
    let mut texts = String::new();

    if let NodeData::Element { ref name, ref attrs, ..  } = node.data {
        if name.local == local_name!("div") && class_attr_eq(&attrs.borrow(), "msg_item") {
            texts.push_str(&parse_message(node));
            return texts;
        }
        if name.local == local_name!("head") {
            return texts;
        }
    }

    for child in node.children.borrow().iter() {
        texts += &retrieve_messages(child.clone());
    }
    let id_regex = Regex::new(r"\[id\d+\|[^\]]+\]").unwrap();
    
    //println!("{}", id_regex.is_match(&texts));
    texts = id_regex.replace_all(&texts, "").into_owned();

    let res = texts.lines().filter_map(|l| match l {
        "" => None,
        line => Some(line.trim())
    }).collect::<Vec<&str>>().join("\n");

    res
}

fn parse_message(node: Handle) -> String {
    let mut msg_body = String::new();

    for child in node.children.borrow().iter() {
        if let NodeData::Element { ref name, ref attrs, .. } = child.data {
            if name.local == local_name!("div") && class_attr_eq(&attrs.borrow(), "msg_body") {
                for body_child in child.children.borrow().iter() {
                    match body_child.data {
                        NodeData::Text { ref contents } => {
                            msg_body += &contents.borrow();
                        },
                        NodeData::Element { ref name, ref attrs, .. } => {
                            if name.local == local_name!("div") && class_attr_eq(&attrs.borrow(), "emoji") {
                                msg_body += &attr_value(&attrs.borrow(), local_name!("alt")).unwrap();
                            }
                            else if name.local == local_name!("br") {
                                msg_body += "\n";
                            }
                        },
                        _ => ()
                    }
                }
            }
        }
    }
    msg_body += "\n";

    msg_body
}

fn class_attr_eq(attrs: &Vec<html5ever::Attribute>, value: &str) -> bool {
    attrs.iter().any(|a| a.name.local == local_name!("class") && *a.value == *value)
}

fn attr_value(attrs: &Vec<html5ever::Attribute>, name: html5ever::LocalName) -> Option<&tendril::StrTendril> {
    attrs.iter().find(|a| a.name.local == name).map(|a| &a.value)
}

