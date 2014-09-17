extern crate string_telephone;
extern crate collections;

use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io;

use string_telephone::client;
use string_telephone::ConnectionConfig;

mod demo_shared;


fn main () {

    let settings = ConnectionConfig {
        protocol_id: 121,
        timeout_period: 10,
        packet_deserializer: demo_shared::deserializer,
        packet_serializer: demo_shared::serializer
    };

    match client::Client::connect(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 0}, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, settings) {
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
