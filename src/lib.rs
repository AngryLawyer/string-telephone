//!
//! # String Telephone
//!
//! Simple abstractions for networking for video games
//!
#![feature(unsafe_destructor)]
#![feature(globs)]
#![feature(macro_rules)]
#![crate_name = "string_telephone"]
#![crate_type="lib"]

extern crate time;

pub use packet::*;
pub use shared::*;
pub use client::*;
pub use server::*;

pub mod packet;
pub mod shared;
pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    mod test_shared;
    mod test_client;
    mod test_server;
}
