use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::io::{IoResult, TimedOut};
use std::comm::{Disconnected, Empty};
use std::collections::TreeMap;
use packet::{Packet, PacketType, PacketConnect, PacketDisconnect, PacketMessage, TaskCommand, Disconnect};
use shared::ConnectionConfig;
use time::now;


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

pub enum PacketOrCommand <T> {
    UserPacket(T),
    Command(PacketType)
}

fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<(Packet, SocketAddr)>, reader_sub_in: Receiver<TaskCommand>, protocol_id: u32) {
    let mut buf = [0, ..1023];
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

fn writer_process(mut writer: UdpSocket, _writer_sub_out: Sender<TaskCommand>, writer_sub_in: Receiver<(Packet, SocketAddr)>) {
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

pub struct Server <T> {
    pub addr: SocketAddr,
    pub config: ConnectionConfig<T>,

    reader_send: Sender<TaskCommand>,
    reader_receive: Receiver<(Packet, SocketAddr)>,
    writer_send: Sender<(Packet, SocketAddr)>,
    writer_receive: Receiver<TaskCommand>,

    connections: TreeMap<String, ClientInstance>
}

impl <T> Server <T> {
    pub fn new(addr: SocketAddr, config: ConnectionConfig<T>) -> IoResult<Server<T>> {
        match UdpSocket::bind(addr) {
            Ok(reader) => {
                let writer = reader.clone();
                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                let protocol_id = config.protocol_id;

                spawn(proc() {
                    reader_process(reader, reader_sub_out, reader_sub_in, protocol_id);
                });

                let (writer_out, writer_sub_in) = channel();
                let (writer_sub_out, writer_in) = channel();

                spawn(proc() {
                    writer_process(writer, writer_sub_out, writer_sub_in);
                });
                
                Ok(Server {
                    addr: addr,
                    config: config,
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

    pub fn poll(&mut self) -> Option<(PacketOrCommand<T>, SocketAddr)> {
        let mut out = None;
        loop {
            match self.reader_receive.try_recv() {
                Ok((packet, src)) => {
                    //Handle any new connections
                    match packet.packet_type {
                        PacketConnect => {
                            self.connections.insert(hash_sender(&src), ClientInstance::new(src, now().to_timespec().sec + self.config.timeout_period.num_seconds()));
                            self.writer_send.send((Packet::accept(self.config.protocol_id), src));
                            out = Some((Command(PacketConnect), src));
                            break
                        },
                        PacketDisconnect => {
                            let hash = hash_sender(&src);
                            if self.connections.contains_key(&hash) {
                                out = Some((Command(PacketConnect), src));
                                self.connections.remove(&hash);
                                break
                            }
                        },
                        PacketMessage => {
                            let hash = hash_sender(&src);
                            match self.connections.find_mut(&hash) {
                                Some(ref mut comms) => {
                                    match (self.config.packet_deserializer)(&packet.packet_content.unwrap()) {
                                        Some(deserialized) => {
                                            out = Some((UserPacket(deserialized), src));
                                            //Update our timeout
                                            comms.timeout = now().to_timespec().sec + self.config.timeout_period.num_seconds();
                                            break
                                        },
                                        _ => ()
                                    }
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

    pub fn cull(&mut self) -> Vec<SocketAddr> {
        let mut keep_alive = TreeMap::new();
        let mut culled = vec![];

        let now = now().to_timespec().sec;

        for (hash, connection) in self.connections.iter() {
            if connection.timeout >= now {
                keep_alive.insert(hash.clone(), connection.clone());
            } else {
                culled.push(connection.addr);
            }
        };

        self.connections = keep_alive;
        culled
    }

    pub fn send_to(&mut self, packet: &T, addr: &SocketAddr) -> bool {
        let hashed = hash_sender(addr);
        match self.connections.find(&hashed) {
            Some(_) => {
                self.writer_send.send((Packet::message(self.config.protocol_id, (self.config.packet_serializer)(packet)), addr.clone()));
                true
            },
            None => false
        }
    }

    pub fn send_to_many(&mut self, packet: &T, addrs: &Vec<SocketAddr>) {
        for addr in addrs.iter() {
            self.send_to(packet, addr);
        }
    }

    pub fn send_to_all(&mut self, packet: &T) {
        for addr in self.connections.clone().values() { //FIXME: Urgh
            self.send_to(packet, &addr.addr);
        }
    }

    /**
     * List all of our current connections
     */
    pub fn all_connections(&self) -> Vec<SocketAddr> {
        self.connections.values().map(|client_instance| { client_instance.addr }).collect()
    }
}

#[unsafe_destructor]
impl <T> Drop for Server <T> {

    fn drop(&mut self) {
        self.reader_send.send(Disconnect);
    }
}
