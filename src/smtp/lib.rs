#[crate_id = "smtp#0.1-pre"];

#[comment = "Rust SMTP client"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

//#[crate_type = "dylib"];
//#[crate_type = "rlib"];

#[deny(non_camel_case_types)];
//#[deny(missing_doc)];

pub mod commands;
pub mod common;
pub mod client;
