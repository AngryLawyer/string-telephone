extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io;

use string_telephone::client;
use string_telephone::packet::{Packet, PacketMessage};

fn main () {
    match client::Client::connect(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 0}, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, 121) {
        Ok(ref mut connection) => {
            println!("Connected!")
            loop {
                loop {
                    match connection.poll() {
                        Some(message) => {
                            match message.packet_type {
                                PacketMessage => {
                                    println!("{}", message.packet_content.into_ascii().into_string())
                                },
                                _ => ()
                            }
                        },
                        None => break
                    }
                };
                let input = io::stdin().read_line()
                                       .ok()
                                       .expect("Failed to read line");

                connection.send(&Packet::message(121, input.into_bytes()));
            }
        },
        Err(e) => {
            println!("Error {}", e)
        }
    };
}
