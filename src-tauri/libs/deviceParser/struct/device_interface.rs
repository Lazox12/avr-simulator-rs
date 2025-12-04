use proc_macro2::{ Span, TokenStream};
use quote::{quote, ToTokens};
use xmltree::Element;
use syn::Ident;
#[derive(Debug)]
pub struct Interface {
    pub name: &'static str,
    pub data_type: &'static str, //todo should be enum
}
impl From<&'static Element> for Interface {
    fn from(x: &'static Element) -> Self {
        Interface{
            name: &x.attributes["name"],
            data_type: &x.attributes["type"],
        }
    }
}
impl ToTokens for Interface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let data_type = &self.data_type;

        tokens.extend(quote! {
            crate::r#struct::device_interface::Interface {
                name: #name.to_string(),
                data_type: #data_type.to_string(),
            }
        });
    }
}