extern crate string_telephone;
extern crate collections;

use collections::str::{Slice, Owned};
use string_telephone::{ConnectionConfig, ClientConnectionConfig, Client};

use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;

fn deserializer(message: &Vec<u8>) -> Option<String> {
    match String::from_utf8_lossy(message.as_slice()) {
        Slice(slice) => Some(slice.to_string()),
        Owned(item) => Some(item)
    }
}

fn serializer(packet: &String) -> Vec<u8> {
    packet.clone().into_bytes()
}

/**
 * Test when there isn't a backend to connect to
 * Connect should return an IOResult stating we can't get there
 */
#[test]
fn connection_ignored() {
    let port = 65000;
    let my_addr = SocketAddr{ ip: Ipv4Addr(0, 0, 0, 0), port: 0 };
    let target_addr = SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: port };
    let settings = ConnectionConfig::new(121, 10, deserializer, serializer);
    let client_settings = ClientConnectionConfig::new(1, Duration::seconds(1));

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => fail!("Reported connected when there is no server!"),
        Err(e) => {
            assert!(e.desc == "Failed to connect")
        }
    };
}

#[test]
fn standard_connection() {
    unimplemented!();
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
