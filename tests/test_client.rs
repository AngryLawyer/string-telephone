#![feature(macro_rules)]
extern crate string_telephone;

use string_telephone::{ConnectionConfig, ClientConnectionConfig, Client, Packet};

use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::net::udp::UdpSocket;
use std::time::duration::Duration;

fn deserializer(message: &Vec<u8>) -> Option<Vec<u8>> {
    Some(message.clone())
}

fn serializer(packet: &Vec<u8>) -> Vec<u8> {
    packet.clone()
}

fn generate_settings(port: u16, protocol_id: u32) -> (SocketAddr, SocketAddr, ConnectionConfig<Vec<u8>>, ClientConnectionConfig) {
    let my_addr = SocketAddr{ ip: Ipv4Addr(0, 0, 0, 0), port: 0 };
    let target_addr = SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: port };
    let settings = ConnectionConfig::new(protocol_id, 10, deserializer, serializer);
    let client_settings = ClientConnectionConfig::new(1, Duration::seconds(1));
    (my_addr, target_addr, settings, client_settings)
}

macro_rules! with_bound_socket(
    ($socket:ident, ($variable:ident)$code:block) => (
        spawn(proc() {
            match UdpSocket::bind($socket) {
                Ok(mut $variable) => $code,
                Err(e) => fail!(e)
            }
        });
    )
)

/**
 * Test when there isn't a backend to connect to
 * Connect should return an IOResult stating we can't get there
 */
#[test]
fn connection_ignored() {
    let port = 65000;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => fail!("Reported connected when there is no server!"),
        Err(e) => {
            assert!(e.desc == "Failed to connect")
        }
    };
}

/**
 * Test a normal connection where the backend replies with an accept
 */
#[test]
fn standard_connection() {
    
    let port = 65001;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        let mut buf = [0, ..255];

        socket.set_timeout(Some(10000));
        let (_, src) = match socket.recv_from(buf) {
            Ok((amt, src)) => (amt, src),
            Err(e) => fail!("Socket didn't get a message")
        };
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src);
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(client) => {
            //Success!
        },
        Err(e) => fail!(e)
    };
}

#[test]
fn connection_rejected() {
    unimplemented!();
}

#[test]
fn connection_different_protocol_id() {
    unimplemented!();
}

#[test]
fn different_request_count() {
    unimplemented!();
}

//TODO: Find a sensible way of testing timeout lengths

#[test]
fn empty_polling() {
    unimplemented!();
}

#[test]
fn single_item_polling() {
    unimplemented!();
}

#[test]
fn multiple_item_polling() {
    unimplemented!();
}

#[test]
fn disconnection() {
    unimplemented!();
}

#[test]
fn timeout() {
    unimplemented!();
}
