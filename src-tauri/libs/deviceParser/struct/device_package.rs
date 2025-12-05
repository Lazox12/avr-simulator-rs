use quote::__private::TokenStream;
use quote::{quote, ToTokens};
use xmltree::Element;
use super::utils::find_childs;

#[derive(Debug)]
pub struct Pinout {
    name: &'static str,
    caption: Option<&'static str>,
    pins: &'static [Pin],
}
impl From<&'static Element> for Pinout {
    fn from(x: &'static Element) -> Self {
        Pinout{
            name: &x.attributes["name"],
            caption: x.attributes.get("caption").map(|x| x.as_str()),
            pins: find_childs(x,"pin").into_iter().map(|x1| Pin::from(x1)).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Pin {
    position:&'static str,
    pad:&'static str,
}
impl From<&'static Element> for Pin {
    fn from(x: &'static Element) -> Self {
        Pin{ 
            position: &x.attributes["position"],
            pad:  &x.attributes["pad"]
        }
    }
}
impl ToTokens for Pinout {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let pins = &self.pins;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_package::Pinout {
                name: #name,
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
                position: #position,
                pad: #pad,
            }
        });
    }
}