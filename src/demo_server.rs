extern crate string_telephone;
use string_telephone::packet::PacketMessage;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

use string_telephone::server;

fn main () {
    match server::ServerManager::new(121, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}) {
        Ok(ref mut server) => {
            loop {
                let messages = server.poll();
                for &(ref packet, ref sender) in messages.iter() {
                    match packet.packet_type {
                        PacketMessage => {
                            server.send_to_all(packet);
                        },
                        _ => ()
                    }
                }
            }
        },
        Err(e) => println!("{}", e)
    }
}
