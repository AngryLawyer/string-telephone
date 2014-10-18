#![feature(macro_rules)]
extern crate string_telephone;

use string_telephone::{ConnectionConfig, ClientConnectionConfig, Server, Packet, PollEmpty, PacketConnect, PacketMessage, PacketDisconnect, PollDisconnected, Command};
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::time::duration::Duration;
use std::io::net::udp::UdpSocket;
use std::io::Timer;

mod test_shared;

macro_rules! with_bound_socket(
    (($variable:ident)$code:block) => (
        spawn(proc() {
            match UdpSocket::bind(SocketAddr{ ip: Ipv4Addr(0, 0, 0, 0), port: 0 }) {
                Ok(mut $variable) => $code,
                Err(e) => fail!(e)
            }
        });
    )
)

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
        Ok(_) => (), //passed
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we get empty polls when we have no clients
 */
#[test]
fn empty_poll() {
    let socket = 64001;
    let (my_addr, settings) = generate_settings(socket, 121);
    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            assert!(server.poll().is_none())
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we reject bad connection attempts
 */
#[test]
fn bad_client_attempt() {
    let socket = 64002;
    let (my_addr, settings) = generate_settings(socket, 121);
    let (tx, rx) = channel();

    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(122).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx.send(());
            });
            rx.recv();
            Timer::new().unwrap().sleep(Duration::seconds(1));
            assert!(server.poll().is_none())
            assert!(server.all_connections().len() == 0);
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we can take a single client
 */
#[test]
fn single_client() {
    let socket = 64003;
    let (my_addr, settings) = generate_settings(socket, 121);
    let (tx, rx) = channel();

    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx.send(());
            });
            rx.recv();
            Timer::new().unwrap().sleep(Duration::seconds(1));
            match server.poll() {
                Some((Command(PacketConnect), _))=> (),
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            }
            assert!(server.all_connections().len() == 1);
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we can take multiple clients
 */
#[test]
fn multiple_clients() {
    let socket = 64004;
    let (my_addr, settings) = generate_settings(socket, 121);
    let (tx, rx) = channel();
    let tx2 = tx.clone();

    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx.send(());
            });
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx2.send(());
            });
            rx.recv();
            rx.recv();
            Timer::new().unwrap().sleep(Duration::seconds(1));
            match server.poll() {
                Some((Command(PacketConnect), _))=> (),
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            };
            match server.poll() {
                Some((Command(PacketConnect), _))=> (),
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            }
            assert!(server.all_connections().len() == 2);
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we can cull old clients
 */
#[test]
fn cull() {
    let socket = 64005;
    let (my_addr, mut settings) = generate_settings(socket, 121);
    settings.timeout_period = Duration::seconds(0);
    let (tx, rx) = channel();
    let tx2 = tx.clone();

    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx.send(());
            });
            with_bound_socket!((socket) {
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                tx2.send(());
            });
            rx.recv();
            rx.recv();
            Timer::new().unwrap().sleep(Duration::seconds(1));
            match server.poll() {
                Some((Command(PacketConnect), _))=> (),
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            };
            match server.poll() {
                Some((Command(PacketConnect), _))=> (),
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            }
            Timer::new().unwrap().sleep(Duration::seconds(1));
            assert!(server.all_connections().len() == 2);
            assert!(server.cull().len() == 2);
            assert!(server.all_connections().len() == 0);
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
}

/**
 * Test we can send to one
 */
#[test]
fn send_to_one() {
    let socket = 64006;
    let (my_addr, settings) = generate_settings(socket, 121);
    let (tx, rx) = channel();

    match Server::new(my_addr, settings) {
        Ok(ref mut server) => {
            with_bound_socket!((socket) {
                socket.set_timeout(Some(1000));
                socket.send_to(Packet::connect(121).serialize().unwrap()[], my_addr).ok().expect("Couldn't send a message");
                let (message, _) = test_shared::get_message(&mut socket); //Should be the Accept message
                let (message, _) = test_shared::get_message(&mut socket); //Should be the Message message

                tx.send(Packet::deserialize(message[]).ok().expect("Couldn't deserialize a message"));
            });
            Timer::new().unwrap().sleep(Duration::seconds(1));
            let source = match server.poll() {
                Some((Command(PacketConnect), source)) => source,
                None => fail!("No result found"),
                _ => fail!("Unexpected poll result")
            };
            let message_out = vec![1,2];
            server.send_to(&message_out, &source);
            let message = rx.recv();
            println!("{}", message.packet_type);
            assert!(message.packet_type == PacketMessage);
            assert!(message.packet_content.unwrap() == message_out);
        },
        Err(t) => fail!("Failed to create a server - {}", t)
    };
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

/**
 * Test we can recieve messages
 */
#[test]
fn receive() {
    unimplemented!()
}

/**
 * Test we can handle client disconnects 
 */
#[test]
fn client_disconnect() {
    unimplemented!()
}
