#![feature(macro_rules)]
extern crate string_telephone;

use string_telephone::{ConnectionConfig, ClientConnectionConfig, Server, Packet, PollEmpty, PacketConnect, PacketMessage, PacketDisconnect, PollDisconnected};
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;

mod test_shared;

fn generate_settings(port: u16, protocol_id: u32) -> (SocketAddr, ConnectionConfig<Vec<u8>>) {
    let my_addr = SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: port };
    let settings = ConnectionConfig::new(protocol_id, Duration::seconds(10), test_shared::deserializer, test_shared::serializer);
    (my_addr, settings)
}

/**
 * Test we can start listening
 */
#[test]
fn create_server() {
    let socket = 64000;
    let (my_addr, settings) = generate_settings(socket, 121);
    match Server::new(my_addr, settings) {
        Ok(server) => (), //passed
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we get empty polls when we have no clients
 */
#[test]
fn empty_poll() {
    unimplemented!()
}

/**
 * Test we reject bad connection attempts
 */
#[test]
fn bad_client_attempt() {
    unimplemented!()
}

/**
 * Test we can take multiple clients
 */
#[test]
fn multiple_clients() {
    unimplemented!()
}

/**
 * Test we can cull old clients
 */
#[test]
fn cull() {
    unimplemented!()
}

/**
 * Test we can send to one
 */
#[test]
fn send_to_one() {
    unimplemented!()
}

/**
 * Test we can't send to folks who aren't connected
 */
#[test]
fn send_to_disconnected() {
    unimplemented!()
}

/**
 * Test we can send to multiple folks
 */
#[test]
fn send_to_many() {
    unimplemented!()
}

/**
 * Test we can send to everyone connected
 */
#[test]
fn send_to_all() {
    unimplemented!()
}
