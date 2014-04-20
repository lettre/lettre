/*!
 * SMTP library
 *
 * For now, contains only a basic and uncomplete SMTP client and some common general functions.
 */

#![crate_id = "smtp#0.1-pre"]

#![comment = "Rust SMTP client"]
#![license = "ASL2"]
#![crate_type = "lib"]

//#[crate_type = "dylib"];
//#[crate_type = "rlib"];

#![deny(non_camel_case_types)]
#![deny(missing_doc)]

#![feature(phase)]
#[phase(syntax, link)] extern crate log;

pub mod commands;
pub mod common;
pub mod client;
