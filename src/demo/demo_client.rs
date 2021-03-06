extern crate string_telephone;
extern crate collections;
extern crate core;

use std::old_io::net::ip::{Ipv4Addr, SocketAddr};
use std::old_io;
use std::time::duration::Duration;
use std::sync::mpsc::{channel};
use std::thread::Thread;

use string_telephone::{ConnectionConfig, ClientConnectionConfig, Client, PollFailResult};

mod demo_shared;

fn main () {

    let settings = ConnectionConfig::new(121, Duration::seconds(10), demo_shared::deserializer, demo_shared::serializer);
    let client_settings = ClientConnectionConfig::new(3, Duration::seconds(5));

    match Client::connect(SocketAddr {ip: Ipv4Addr(0, 0, 0, 0), port: 0}, SocketAddr {ip: Ipv4Addr(127, 0, 0, 1), port: 6666}, settings, client_settings) {
        Ok(ref mut connection) => {
            println!("Connected!");

            let (send, recv) = channel();

            Thread::spawn(move || {
                loop {
                    let input = old_io::stdin().read_line()
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
                    Err(PollFailResult::Disconnected) => {
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
