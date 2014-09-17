extern crate collections;
extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

use string_telephone::{ConnectionConfig, Server, UserPacket};

mod demo_shared;

fn main () {
    let settings = ConnectionConfig {
        protocol_id: 121,
        timeout_period: 10,
        packet_deserializer: demo_shared::deserializer,
        packet_serializer: demo_shared::serializer
    };

    match Server::new(SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, settings) {
        Ok(ref mut server) => {
            loop {
                loop {
                    match server.poll() {
                        Some((UserPacket(packet), _)) => {
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
