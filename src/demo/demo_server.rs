extern crate collections;
extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

use string_telephone::server;

mod demo_shared;

fn main () {
    match server::ServerManager::new(121, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, 10, demo_shared::deserializer, demo_shared::serializer) {
        Ok(ref mut server) => {
            loop {
                loop {
                    match server.poll() {
                        Some((server::UserPacket(packet), _)) => {
                            server.send_to_all(&packet);
                        },
                        Some(_) => (),
                        None => break
                    }
                };
                let culled = server.cull();
                if culled.len() > 0 {
                    println!("{}", culled);
                }
            }
        },
        Err(e) => println!("{}", e)
    }
}
