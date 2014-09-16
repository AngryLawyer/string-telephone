use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::io::{IoResult, IoError, OtherIoError, TimedOut};
use std::io::Timer;
use std::comm::{Disconnected, Empty, Select};
use std::time::duration::Duration;
use std::collections::TreeMap;
use packet::{Packet, PacketType, PacketConnect, PacketDisconnect, PacketMessage, PacketAccept, PacketReject, Command, Disconnect};
use time;


//FIXME: Ew ew ew
fn hash_sender(address: &SocketAddr) -> String {
    match address.ip {
        Ipv4Addr(a, b, c, d) => {
            format!("{}-{}-{}-{}:{}", a, b, c, d, address.port)
        },
        Ipv6Addr(a, b, c, d, e, f, g, h) => {
            format!("{}-{}-{}-{}-{}-{}-{}-{}:{}", a, b, c, d, e, f, g, h, address.port)
        }
    }
}

#[deriving(Clone)]
struct ClientInstance {
    addr: SocketAddr,
    timeout: i64
}

impl ClientInstance {
    pub fn new(addr: SocketAddr, timeout: i64) -> ClientInstance {
        ClientInstance {
            addr: addr,
            timeout: timeout
        }
    }
}

/**
 * What we want:
 * Server sits and loops about. Has a send/receive buffer
 * Has a broadcast method?
 * Has a send method
 * Has a read method, to pump items out of the receive buffer
 * Has a read iterator?
 */
fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<(Packet, SocketAddr)>, reader_sub_in: Receiver<Command>, protocol_id: u32) {
    let mut buf = [0, ..255];
    reader.set_timeout(Some(1000));
    loop {
        match reader.recv_from(buf) {
            Ok((amt, src)) => {
                match Packet::deserialize(buf.slice_to(amt)) {
                    Ok(packet) => {
                        if packet.protocol_id == protocol_id {
                            reader_sub_out.send((packet, src));
                        }
                    },
                    Err(_) => ()
                }
            },
            Err(e) => {
                match e.kind {
                    TimedOut => {
                        match reader_sub_in.try_recv() {
                            Ok(Disconnect) => {
                                break;
                            },
                            Err(Disconnected) => {
                                break;
                            },
                            Err(Empty) => ()
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}

fn writer_process(mut writer: UdpSocket, writer_sub_out: Sender<Command>, writer_sub_in: Receiver<(Packet, SocketAddr)>) {
    for (msg, target_addr) in writer_sub_in.iter() {
        match msg.serialize() {
            Ok(msg) => {
                match writer.send_to(msg.as_slice(), target_addr) {
                    Ok(()) => (),
                    Err(e) => println!("Error sending data - {}", e)
                }
            },
            Err(_) => ()
        }
    }
}

pub struct ServerManager {
    pub addr: SocketAddr,

    protocol_id: u32,
    reader_send: Sender<Command>,
    reader_receive: Receiver<(Packet, SocketAddr)>,
    writer_send: Sender<(Packet, SocketAddr)>,
    writer_receive: Receiver<Command>,

    connections: TreeMap<String, ClientInstance>
}

impl ServerManager {
    pub fn new(protocol_id: u32, addr: SocketAddr) -> IoResult<ServerManager> {
        match UdpSocket::bind(addr) {
            Ok(reader) => {
                let writer = reader.clone();
                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                spawn(proc() {
                    reader_process(reader, reader_sub_out, reader_sub_in, protocol_id);
                });

                let (writer_out, writer_sub_in) = channel();
                let (writer_sub_out, writer_in) = channel();

                spawn(proc() {
                    writer_process(writer, writer_sub_out, writer_sub_in);
                });
                
                Ok(ServerManager {
                    protocol_id: protocol_id,
                    addr: addr,
                    reader_send: reader_out,
                    reader_receive: reader_in,
                    writer_send: writer_out,
                    writer_receive: writer_in,
                    connections: TreeMap::new()
                })
            }
            Err(e) => Err(e)
        }
    }

    pub fn poll(&mut self) -> Option<(Packet, SocketAddr)> {
        let mut out = None;
        loop {
            match self.reader_receive.try_recv() {
                Ok((packet, src)) => {
                    //Handle any new connections
                    match packet.packet_type {
                        PacketConnect => {
                            self.connections.insert(hash_sender(&src), ClientInstance::new(src, time::now().to_timespec().sec + 10)); //FIXME: Shouldn't be done here
                            self.writer_send.send((Packet::accept(self.protocol_id), src));
                            out = Some((packet, src));
                            break
                        },
                        PacketDisconnect => {
                            let hash = hash_sender(&src);
                            if self.connections.contains_key(&hash) {
                                out = Some((packet, src));
                                self.connections.remove(&hash);
                                break
                            }
                        },
                        PacketMessage => {
                            let hash = hash_sender(&src);
                            match self.connections.find_mut(&hash) {
                                Some(ref mut comms) => {
                                    out = Some((packet, src));
                                    //Update our timeout
                                    comms.timeout = time::now().to_timespec().sec + 10; //FIXME: Stop hardcoding
                                    break
                                },
                                None => ()
                            }
                        },
                        _ => ()
                    };
                },
                _ => {
                    break
                }
            };
        };
        out
    }

    pub fn cull(&mut self) {
        let mut keep_alive = TreeMap::new();
        let now = time::now().to_timespec().sec;

        for (hash, connection) in self.connections.iter() {
            if connection.timeout >= now {
                keep_alive.insert(hash.clone(), connection.clone());
            }
        };

        self.connections = keep_alive;
    }

    pub fn send_to(&mut self, packet: &Packet, addr: &SocketAddr) {
        let hashed = hash_sender(addr);
        match self.connections.find(&hashed) {
            Some(_) => self.writer_send.send((packet.clone(), addr.clone())),
            None => (),
        }
    }

    pub fn send_to_many(&mut self, packet: &Packet, addrs: &Vec<SocketAddr>) {
        for addr in addrs.iter() {
            self.send_to(packet, addr)
        }
    }

    pub fn send_to_all(&mut self, packet: &Packet) {
        for addr in self.connections.clone().values() { //FIXME: Urgh
            self.send_to(packet, &addr.addr)
        }
    }
}

impl Drop for ServerManager {

    fn drop(&mut self) {
        self.reader_send.send(Disconnect);
    }
}
