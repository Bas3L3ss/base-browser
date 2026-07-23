use std::println;

pub mod parser;
pub mod tokenizer;

pub fn print_dom_tree(nodes: &Vec<crate::parser::node::Node>) {
    fn print_node(nodes: &Vec<crate::parser::node::Node>, node_id: usize, indent: usize) {
        // Increase indentation by making it 4 spaces per level instead of 2
        let indent_str = "    ".repeat(indent);
        if let Some(node) = nodes.get(node_id) {
            use crate::parser::node_type::NodeType;
            match &node.ntype {
                NodeType::Document => println!("{indent_str}Document"),
                NodeType::Doctype(s) => println!("{indent_str}<!DOCTYPE {s}>"),
                NodeType::Element(el) => {
                    println!("{indent_str}<{:?}>", el.tag);
                    for child_id in &node.children {
                        print_node(nodes, *child_id, indent + 1);
                    }
                    println!("{indent_str}</{:?}>", el.tag);
                    return;
                }
                NodeType::Text(t) => println!("{indent_str}\"{t}\""),
                NodeType::Comment(c) => println!("{indent_str}<!-- {c} -->"),
            }
            for child_id in &node.children {
                print_node(nodes, *child_id, indent + 1);
            }
        }
    }
    for (i, node) in nodes.iter().enumerate() {
        if node.parent.is_none() {
            print_node(nodes, i, 0);
        }
    }
}
