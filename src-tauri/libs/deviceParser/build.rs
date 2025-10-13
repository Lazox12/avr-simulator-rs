use std::cell::Ref;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs;
use std::ops::{Deref, DerefMut};
use build_print::{println, *};
use quick_xml::events::attributes::{Attribute, Attributes};
use quick_xml::name::QName;

#[derive(Debug, Clone)]
struct Tree {
    name: String,
    attributes: Vec<u8>,
    inner: Vec<Tree>,
    parent: Option<Box<Tree>>,
}
impl AsRef<Tree> for Tree {
    fn as_ref(&self) -> &Self {  // Self is Struct<'a>, the type for which we impl AsRef
        self
    }
}
fn main() {
    let file = std::fs::read_dir("atdf").unwrap();
    file.into_iter().for_each(|entry| {
        let mut base_tree:Option<Tree> =None;
        let mut tree_ref:Option<&mut Tree> =None;
        let text = fs::read_to_string(&entry.unwrap().path()).unwrap();
        let mut reader = Reader::from_str(&*text);
        reader.config_mut().trim_text(true);
        loop{
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    let data = e;
                    let node = Tree{name :data.name().into_inner().to_vec().iter().map(|x| *x as char).collect(),
                        attributes: data.attributes()
                            .map(|a| a.unwrap().value.as_ref().to_vec())
                            .flatten()
                            .collect(),
                        inner:Vec::new(),
                        parent:match tree_ref {
                            None => None,
                            Some(tree)=>{
                                Some(Box::new(tree_ref))
                            }
                        },
                    };

                    if let Some(tree) = tree_ref.as_deref_mut() {
                        tree.inner.push(node);
                    } else {
                        base_tree = Some(node);
                        tree_ref = base_tree.as_mut();
                    }
                }
                Ok(Event::End(e)) => {
                    info!("{:?}",e)
                }
                Ok(Event::Empty(e)) => {
                    info!("{:?}",e)
                }
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),

                Ok(e) => {
                    warn!("invalid branch: {:?}",e);
                }
            }
        }
        return;
    })
}