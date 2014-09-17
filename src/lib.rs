#![feature(unsafe_destructor)]
#![crate_name = "string_telephone"]
#![crate_type="lib"]

extern crate time;

pub mod packet;
pub mod shared;
pub mod client;
pub mod server;
