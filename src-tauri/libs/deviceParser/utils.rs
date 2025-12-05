use std::time::{Duration, Instant};
use proc_macro2::{Span};
use xmltree::{Element, XMLNode};
use syn::Ident;

pub fn find_child(e:&'static Element, name:&str) ->Option<&'static Element>{
    e.children.iter().find_map(|x| {match x {
        XMLNode::Element(e) =>if e.name == name { Some(e) } else { None },
        _ => None
    }})
}
pub fn find_childs(e:&'static Element,name:&str)->Vec<&'static Element>{
    e.children.iter().filter_map(|x| {match x {
        XMLNode::Element(e) =>if e.name == name { Some(e) } else { None },
        _ => None
    }}).collect()
}

pub fn time_function<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}
pub fn to_ident(s: &str) -> Ident {
    // Replace invalid characters like '-' with '_'
    let sanitized = s.replace("-", "_").replace(" ", "_");
    // If it starts with a number, prefix it (Rust identifiers can't start with numbers)
    let final_name = if sanitized.chars().next().unwrap().is_numeric() {
        format!("_{}", sanitized)
    } else {
        sanitized
    };
    Ident::new(&final_name, Span::call_site())
}