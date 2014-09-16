extern crate string_telephone;
extern crate collections;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io;
use collections::str::{Slice, Owned};

use string_telephone::client;

fn deserializer(message: &Vec<u8>) -> String {
    match String::from_utf8_lossy(message.as_slice()) {
        Slice(slice) => slice.to_string(),
        Owned(item) => item
    }
}

fn serializer(packet: &String) -> Vec<u8> {
    packet.clone().into_bytes()
}

fn main () {

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
                        println!("{}", message);
                    },
                    Err(client::PollDisconnected) => {
                        println!("Timed out");
                        break
                    },
                    _ => ()
                };
                
                match recv.try_recv() {
                    Ok(text) => {
                        connection.send(&text);
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
