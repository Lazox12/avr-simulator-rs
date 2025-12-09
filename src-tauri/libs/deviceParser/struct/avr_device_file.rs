use proc_macro2::{Span};
use syn::Ident;
use quote::__private::TokenStream;
use quote::{quote, ToTokens};
use xmltree::Element;
use super::utils::{find_child, find_childs};
use super::device_info::{Device, Variant};
use super::device_package::Pinout;
use super::module::Module;

#[derive(Debug,Default)]
pub struct AvrDeviceFile {
    pub variants:&'static [Variant],
    pub devices:Device,
    pub modules:&'static [Module],
    pub pinouts:Option<&'static [Pinout]>,
}
impl From<&'static Element> for AvrDeviceFile {
    fn from(element:&'static Element) -> Self {
        AvrDeviceFile{
            variants: Box::leak(find_childs(find_child(element,"variants").unwrap(),"variant").into_iter().map(|x| {Variant::from(x)}).collect::<Vec<Variant>>().into_boxed_slice()),
            devices: find_child(find_child(element,"devices").unwrap(),"device").map(|f| Device::from(f)).unwrap(),
            modules: Box::leak(find_childs(find_child(element,"modules").unwrap(),"module").into_iter().map(|x| {Module::from(x)}).collect::<Vec<Module>>().into_boxed_slice()),
            pinouts: match find_child(element,"pinouts").map(|x| find_childs(x,"pinout").into_iter().map(|x| {Pinout::from(x)}).collect::<Vec<Pinout>>().into_boxed_slice()) {
                None => {None}
                Some(x) => {Some(Box::leak(x))}
            },
        }
    }
}
impl ToTokens for AvrDeviceFile {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = &self.variants;
        let devices = &self.devices;
        let modules = &self.modules;

        let pinouts = match &self.pinouts {
            Some(p) => quote! { Some(&[#( #p ),*]) },
            None => quote! { None },
        };

        // Convert name to Ident to remove quotes
        let name_str = self.devices.name.replace("-", "_").to_uppercase();
        let name_ident = Ident::new(&name_str, Span::call_site());

        let output = quote! {
            pub const #name_ident: crate::r#struct::avr_device_file::AvrDeviceFile = crate::r#struct::avr_device_file::AvrDeviceFile {
                variants: &[#( #variants ),*],
                devices: #devices,
                modules: &[#( #modules ),*],
                pinouts: #pinouts,
            };
        };

        tokens.extend(output);
    }
}