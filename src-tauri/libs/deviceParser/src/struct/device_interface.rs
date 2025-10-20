use xmltree::Element;

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub data_type: String, //todo should be enum
}
impl From<&Element> for Interface {
    fn from(x: &Element) -> Self {
        Interface{
            name: x.attributes["name"].to_string(),
            data_type: x.attributes["type"].to_string(),
        }
    }
}