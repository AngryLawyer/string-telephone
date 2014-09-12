extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

use string_telephone::server;

fn main () {
    match server::ServerManager::new(121, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}) {
        Ok(ref mut server) => {
            loop {
                server.poll();
            }
        },
        Err(e) => println!("{}", e)
    }
}
