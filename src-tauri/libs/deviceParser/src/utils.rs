use std::time::{Duration, Instant};
use xmltree::{Element, XMLNode};

pub fn find_child(e:&Element, name:String) ->Option<&Element>{
    e.children.iter().find_map(|x| {match x {
        XMLNode::Element(e) =>if e.name == name { Some(e) } else { None },
        _ => None
    }})
}
pub fn find_childs(e:&Element,name:String)->Vec<&Element>{
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