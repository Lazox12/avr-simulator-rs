use xmltree::Element;
use crate::r#struct::module::Value;
use crate::utils::find_childs;

#[derive(Debug)]
pub struct Pinout {
    name: String,
    caption: Option<String>,
    pins: Vec<Pin>,
}
impl From<&Element> for Pinout {
    fn from(x: &Element) -> Self {
        Pinout{
            name: x.attributes["name"].to_string(),
            caption: x.attributes.get("caption").map(|x| x.to_string()),
            pins: find_childs(x,"pin".to_string()).into_iter().map(|x1| {Pin::from(x1)}).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Pin {
    position:String,
    pad:String,
}
impl From<&Element> for Pin {
    fn from(x: &Element) -> Self {
        Pin{ 
            position: x.attributes["position"].to_string(),
            pad:  x.attributes["pad"].to_string()
        }
    }
}