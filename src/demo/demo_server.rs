extern crate collections;
extern crate string_telephone;
extern crate core;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;

use string_telephone::{ConnectionConfig, Server, PacketOrCommand};

mod demo_shared;

fn main () {
    let settings = ConnectionConfig::new(121, Duration::seconds(10), demo_shared::deserializer, demo_shared::serializer);

    match Server::new(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 6666}, settings) {
        Ok(ref mut server) => {
            loop {
                loop {
                    match server.poll() {
                        Some((PacketOrCommand::UserPacket(packet), _)) => {
                            server.send_to_all(&packet);
                        },
                        Some(_) => {
                            println!("PACKET");
                        },
                        None => break
                    }
                };
                let culled = server.cull();
                if culled.len() > 0 {
                    let mapped: Vec<String> = culled.iter().map(|addr| { format!("{}", addr) }).collect();
                    println!("{}", mapped.connect(","))
                }
            }
        },
        Err(e) => println!("{}", e)
    }
}
