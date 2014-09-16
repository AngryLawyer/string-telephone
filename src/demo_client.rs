extern crate string_telephone;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io;

use string_telephone::client;
use string_telephone::packet::{Packet, PacketMessage};

fn main () {
    let deserializer = |addr: Option<SocketAddr>, message: &Vec<u8>| -> Vec<u8> {
        message.clone()
    };

    let serializer = |packet: &Vec<u8>| -> Vec<u8> {
        packet.clone()
    };

    match client::Client::connect(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 0}, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, 121, 10, deserializer, serializer) {
        Ok(ref mut connection) => {
            println!("Connected!")

            let (send, recv) = channel();

            spawn(proc() {
                loop {
                    let input = io::stdin().read_line()
                                           .ok()
                                           .expect("Failed to read line");

                    send.send(input);
                }
            });

            loop {
                match connection.poll() {
                    Ok(message) => {
                        match message.packet_type {
                            PacketMessage => {
                                println!("{}", message.packet_content.unwrap().into_ascii().into_string())
                            },
                            _ => ()
                        }
                    },
                    Err(client::PollDisconnected) => {
                        println!("Timed out");
                        break
                    },
                    _ => ()
                };
                
                match recv.try_recv() {
                    Ok(text) => {
                        connection.send(&Packet::message(121, text.into_bytes()));
                    },
                    Err(_) => ()
                }
            }
        },
        Err(e) => {
            println!("Error {}", e)
        }
    };
}
