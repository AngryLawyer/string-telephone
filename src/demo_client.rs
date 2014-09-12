extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

use string_telephone::client;

fn main () {
    match client::Client::connect(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 0}, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, 121) {
        Ok(connection) => {
            println!("Hola!")
        },
        Err(e) => {
            println!("Error {}", e)
        }
    };
}
