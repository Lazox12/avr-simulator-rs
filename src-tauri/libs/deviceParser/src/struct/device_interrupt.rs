use xmltree::Element;

#[derive(Debug)]
pub struct Interrupt{
    pub index:i64,
    pub name:String,
    pub caption:Option<String>,
}
impl From<&Element> for Interrupt{
    fn from(x:&Element) -> Interrupt{
        Interrupt{
            index: x.attributes["index"].to_string().parse().unwrap(),
            name: x.attributes["name"].to_string(),
            caption: x.attributes.get("caption").map(|x| x.to_string()),
        }
    }
}