use proc_macro2::{ Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use syn::Ident;
use crate::utils::to_ident;

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
impl ToTokens for Interrupt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let index = self.index;
        let name = &self.name;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c.to_string()) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_interrupt::Interrupt {
                index: #index,
                name: #name.to_string(),
                caption: #caption,
            }
        });
    }
}