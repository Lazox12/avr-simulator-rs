use quote::__private::TokenStream;
use quote::{quote, ToTokens};
use xmltree::Element;
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
impl ToTokens for Pinout {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let pins = &self.pins;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_package::Pinout {
                name: #name.to_string(),
                caption: #caption,
                pins: vec![#( #pins ),*],
            }
        });
    }
}

impl ToTokens for Pin {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let position = &self.position;
        let pad = &self.pad;

        tokens.extend(quote! {
            crate::r#struct::device_package::Pin {
                position: #position.to_string(),
                pad: #pad.to_string(),
            }
        });
    }
}