use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use xmltree::Element;

#[derive(Debug)]
pub struct Interrupt{
    pub index:i64,
    pub name:&'static str,
    pub caption:Option<&'static str>,
}
impl From<&'static Element> for Interrupt{
    fn from(x:&'static Element) -> Interrupt{
        Interrupt{
            index: x.attributes["index"].parse().unwrap(),
            name: &x.attributes["name"],
            caption: x.attributes.get("caption").map(|x| x.as_str()),
        }
    }
}
impl ToTokens for Interrupt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let index = self.index;
        let name = &self.name;
        let caption = match &self.caption {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };

        tokens.extend(quote! {
            crate::r#struct::device_interrupt::Interrupt {
                index: #index,
                name: #name,
                caption: #caption,
            }
        });
    }
}