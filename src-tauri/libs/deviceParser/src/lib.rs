#[path = "../struct/mod.rs"]
pub mod r#struct;

#[path = "../utils.rs"]
pub mod utils;

include!(concat!(env!("OUT_DIR"), "/avr/mod.rs"));