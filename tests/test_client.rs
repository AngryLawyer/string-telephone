#![feature(macro_rules)]
extern crate string_telephone;

use string_telephone::{ConnectionConfig, ClientConnectionConfig, Client, Packet, PollEmpty, PacketConnect, PacketMessage, PacketDisconnect, PollDisconnected};

use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::net::udp::UdpSocket;
use std::io::Timer;
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
    let client_settings = ClientConnectionConfig::new(3, Duration::seconds(2));
    (my_addr, target_addr, settings, client_settings)
}

fn get_message(socket: &mut UdpSocket) -> (Vec<u8>, SocketAddr) {
    let mut buf = [0, ..255];
    match socket.recv_from(buf) {
        Ok((amt, src)) => (buf.slice_to(amt).to_vec(), src),
        Err(e) => fail!("Socket didn't get a message - {}", e)
    }
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
        socket.set_timeout(Some(1000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Failed to send accept packet");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => {
            //Success!
        },
        Err(e) => fail!(e)
    };
}

/**
 * If the server gives us back a different protocol id, we shouldn't accept it
 */
#[test]
fn connection_different_protocol_id() {
    let port = 65002;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(1000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(122).serialize().unwrap()[], src).ok().expect("Failed to send accept packet");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => fail!("Connected to a server with a different protocol ID!"),
        Err(e) => {
            assert!(e.desc == "Failed to connect")
        }
    };
}

/**
 * If the server specifically rejects us, we should give up
 */
#[test]
fn connection_rejected() {
    let port = 65003;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(1000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::reject(121).serialize().unwrap()[], src).ok().expect("Failed to send reject packet");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => fail!("Connected to a server that rejected us!"),
        Err(e) => {
            assert!(e.desc == "Failed to connect")
        }
    };
}


/**
 * We should be able to specify how many retries we want
 */
#[test]
fn different_retry_count() {
    let port = 65004;
    let (my_addr, target_addr, settings, mut client_settings) = generate_settings(port, 121);
    client_settings.max_connect_retries = 6;
    client_settings.connect_attempt_timeout = Duration::milliseconds(100);

    let (tx, rx) = channel();
    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let mut attempts = 0u8;
        while attempts < 3 {
            get_message(&mut socket);
            attempts += 1;
        }

        tx.send(attempts);
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => (),
        Err(_) => ()
    };

    assert!(rx.recv() == 3);
}

//TODO: Find a sensible way of testing timeout lengths

/**
 * We should instantly return when polling and there's no data available
 */
#[test]
fn empty_polling() {
    let port = 65005;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(1000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Failed to send accept packet");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            assert!(match client.poll() { Err(PollEmpty) => true, _ => false});
        },
        Err(e) => fail!(e)
    };
}

/**
 * We should return an item if there's one there
 */
#[test]
fn single_item_polling() {
    let port = 65006;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![1]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            //May have to wait a bit
            //FIXME: There must be a better way of doing this
            Timer::new().unwrap().sleep(Duration::seconds(1));
            match client.poll() { 
                Ok(packet) => {
                    assert!(packet == vec![1]);
                },
                Err(e) => fail!("Couldn't match a polled message! - {}", e)
            };
        },
        Err(e) => fail!(e)
    };
}

/**
 * We should be able to pump multiple items
 */
#[test]
fn multiple_item_polling() {
    let port = 65007;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![1]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![2]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![3]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
    });

    let mut packets: Vec<Vec<u8>> = vec![];

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            //FIXME: There must be a better way of doing this
            Timer::new().unwrap().sleep(Duration::seconds(1));
            loop {
                match client.poll() { 
                    Ok(packet) => packets.push(packet),
                    Err(PollEmpty) => break,
                    Err(e) => fail!("Unexpected failure - {}", e)
                };
            }
        },
        Err(e) => fail!("{}", e)
    };

    assert!(packets.len() == 3);
    assert!(packets[0] == vec![1]);
    assert!(packets[1] == vec![2]);
    assert!(packets[2] == vec![3]);
}

/**
 * Messages not intended for us shouldn't be forwarded to us
 */
#[test]
fn ignore_bad_queue_items_polling() {
    let port = 65008;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![1]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(122, vec![2]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::message(121, vec![3]).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
    });

    let mut packets: Vec<Vec<u8>> = vec![];

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            //FIXME: There must be a better way of doing this
            Timer::new().unwrap().sleep(Duration::seconds(1));
            loop {
                match client.poll() { 
                    Ok(packet) => packets.push(packet),
                    Err(PollEmpty) => break,
                    Err(e) => fail!("Unexpected failure - {}", e)
                };
            }
        },
        Err(e) => fail!("{}", e)
    };

    assert!(packets.len() == 2);
    assert!(packets[0] == vec![1]);
    assert!(packets[1] == vec![3]);
}

/**
 * If the other end tells us to hang up, we better listen
 */
#[test]
fn disconnection() {
    let port = 65008;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        socket.send_to(Packet::disconnect(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            //FIXME: There must be a better way of doing this
            Timer::new().unwrap().sleep(Duration::seconds(1));
            loop {
                match client.poll() { 
                    Err(PollDisconnected) => break,
                    _ => fail!("Unexpected failure")
                };
            }
        },
        Err(e) => fail!("{}", e)
    };
}

/**
 * If the remote end doesn't reply in time, we should time out and disconnect
 */
#[test]
fn timeout() {
    let port = 65009;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        //Don't send any more data
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut client) => {
            //FIXME: There must be a better way of doing this
            Timer::new().unwrap().sleep(Duration::seconds(1));
            loop {
                match client.poll() { 
                    Err(PollDisconnected) => break,
                    Err(PollEmpty) => (),
                    _ => fail!("Unexpected result")
                };
            }
        },
        Err(e) => fail!("{}", e)
    };
}

/**
 * When trying to connect, we should send a connect request message
 */
#[test]
fn send_correct_handshake() {
    let port = 65010;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    let (tx, rx) = channel();

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (msg, src) = get_message(&mut socket);
        //Check what's been sent
        let packet = Packet::deserialize(msg[]);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        tx.send(packet);
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => (),
        Err(_) => ()
    };

    let packet = rx.recv().unwrap();
    assert!(packet.protocol_id == 121);
    assert!(packet.packet_type == PacketConnect);
    assert!(packet.packet_content.is_none())
}

/**
 * We should be able to send data that can be read by the server
 */
#[test]
fn send_data() {
    let port = 65011;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    let (tx, rx) = channel();

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        //Check what's been sent
        let (msg, _) = get_message(&mut socket);
        let packet = Packet::deserialize(msg[]);
        tx.send(packet);
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(ref mut socket) => {
            socket.send(&vec![1, 2, 3]);
        },
        Err(_) => ()
    };

    let packet = rx.recv().unwrap();
    assert!(packet.protocol_id == 121);
    assert!(packet.packet_type == PacketMessage);
    assert!(packet.packet_content.unwrap() == vec![1, 2, 3])
}

/**
 * When the connection deconstructs, we should tell the server
 */
#[test]
fn client_disconnect() {
    let port = 65012;
    let (my_addr, target_addr, settings, client_settings) = generate_settings(port, 121);

    let (tx, rx) = channel();

    with_bound_socket!(target_addr, (socket) {
        socket.set_timeout(Some(10000));
        let (_, src) = get_message(&mut socket);
        socket.send_to(Packet::accept(121).serialize().unwrap()[], src).ok().expect("Couldn't send a message");
        //Check what's been sent
        let (msg, _) = get_message(&mut socket);
        let packet = Packet::deserialize(msg[]);
        tx.send(packet);
    });

    match Client::connect(my_addr, target_addr, settings, client_settings) {
        Ok(_) => (),
        Err(_) => ()
    };

    let packet = rx.recv().unwrap();
    assert!(packet.protocol_id == 121);
    assert!(packet.packet_type == PacketDisconnect);
    assert!(packet.packet_content.is_none())
}
