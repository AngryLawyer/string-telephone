use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::io::{IoResult, TimedOut};
use std::sync::mpsc::{Sender, Receiver, TryRecvError, channel, Select};
use std::thread::Thread;
use std::collections::BTreeMap;
use packet::{Packet, PacketType, TaskCommand};
use shared::{ConnectionConfig, SequenceManager};
use time::now;


//FIXME: Ew ew ew - there must be a nicer way of hashing
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
    timeout: i64,
    sequence_manager: SequenceManager
}

impl ClientInstance {
    pub fn new(addr: SocketAddr, timeout: i64) -> ClientInstance {
        ClientInstance {
            addr: addr,
            timeout: timeout,
            sequence_manager: SequenceManager::new()
        }
    }

    pub fn next_sequence_id(&mut self) -> u16 {
        self.sequence_manager.next_sequence_id()
    }
}

/**
 * Types of packet we can receive as a server
 */
pub enum PacketOrCommand <T> {
    ///A message packet, containing whichever type we're set up to handle
    UserPacket(T),
    ///An internal control packet
    Command(PacketType)
}

fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<(Packet, SocketAddr)>, reader_sub_in: Receiver<TaskCommand>, protocol_id: u32) {
    let mut buf = [0; 1024];
    reader.set_timeout(Some(1000));
    loop {
        match reader.recv_from(&mut buf) {
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
                            Ok(TaskCommand::Disconnect) => {
                                break;
                            },
                            Err(TryRecvError::Disconnected) => {
                                break;
                            },
                            Err(TryRecvError::Empty) => ()
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

/**
 * A UDP server, which manages multiple clients
 */
pub struct Server <T> {
    ///Which address to listen on
    pub addr: SocketAddr,
    ///Basic configuration for the server
    pub config: ConnectionConfig<T>,

    reader_send: Sender<TaskCommand>,
    reader_receive: Receiver<(Packet, SocketAddr)>,
    writer_send: Sender<(Packet, SocketAddr)>,

    connections: BTreeMap<String, ClientInstance>
}

impl <T> Server <T> {
    /**
     * Start listening on a given socket
     */
    pub fn new(addr: SocketAddr, config: ConnectionConfig<T>) -> IoResult<Server<T>> {
        match UdpSocket::bind(addr) {
            Ok(reader) => {
                let writer = reader.clone();
                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                let protocol_id = config.protocol_id;

                Thread::spawn(move || {
                    reader_process(reader, reader_sub_out, reader_sub_in, protocol_id);
                });

                let (writer_out, writer_sub_in) = channel();
                let (writer_sub_out, _) = channel();

                Thread::spawn(move || {
                    writer_process(writer, writer_sub_out, writer_sub_in);
                });
                
                Ok(Server {
                    addr: addr,
                    config: config,
                    reader_send: reader_out,
                    reader_receive: reader_in,
                    writer_send: writer_out,
                    connections: BTreeMap::new()
                })
            }
            Err(e) => Err(e)
        }
    }

    /**
     * Pump any messages that have been sent to us
     *
     * Note that this internally accepts connections and disconnects, with the original packets being returned
     */
    pub fn poll(&mut self) -> Option<(PacketOrCommand<T>, SocketAddr)> {
        let mut out = None;
        loop {
            match self.reader_receive.try_recv() {
                Ok((packet, src)) => {
                    //Handle any new connections
                    match packet.packet_type {
                        PacketType::Connect => {
                            let hash = hash_sender(&src);
                            //Don't accept multiple connection attempts from the same client
                            if self.connections.contains_key(&hash) == false {
                                self.connections.insert(hash_sender(&src), ClientInstance::new(src, now().to_timespec().sec + self.config.timeout_period.num_seconds()));
                                self.writer_send.send((Packet::accept(self.config.protocol_id, 0), src));
                                out = Some((PacketOrCommand::Command(PacketType::Connect), src));
                            }
                            break
                        },
                        PacketType::Disconnect => {
                            let hash = hash_sender(&src);
                            if self.connections.contains_key(&hash) {
                                out = Some((PacketOrCommand::Command(PacketType::Disconnect), src));
                                self.connections.remove(&hash);
                                break
                            }
                        },
                        PacketType::Message => {
                            let hash = hash_sender(&src);
                            match self.connections.get_mut(&hash) {
                                Some(comms) => {
                                    if comms.sequence_manager.packet_is_newer(packet.sequence_id) {
                                        comms.sequence_manager.set_newest_packet(packet.sequence_id);
                                        match (self.config.packet_deserializer)(&packet.packet_content.unwrap()) {
                                            Some(deserialized) => {
                                                out = Some((PacketOrCommand::UserPacket(deserialized), src));
                                                //Update our timeout
                                                comms.timeout = now().to_timespec().sec + self.config.timeout_period.num_seconds();
                                                break
                                            },
                                            _ => ()
                                        }
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

    /**
     * Disconnect, and return, any sockets that have not contacted us for our timeout duration
     */
    pub fn cull(&mut self) -> Vec<SocketAddr> {
        let mut keep_alive = BTreeMap::new();
        let mut culled = vec![];

        let now = now().to_timespec().sec;

        for (hash, connection) in self.connections.into_iter() {
            if connection.timeout >= now {
                keep_alive.insert(hash, connection);
            } else {
                culled.push(connection.addr);
            }
        };

        self.connections = keep_alive;
        culled
    }

    /**
     * Send a packet to a specific address
     *
     * This will return false if the given address isn't connected to us
     */
    pub fn send_to(&mut self, packet: &T, addr: &SocketAddr) -> bool {
        let hashed = hash_sender(addr);
        match self.connections.get_mut(&hashed) {
            Some(comms) => {
                self.writer_send.send((Packet::message(self.config.protocol_id, comms.next_sequence_id(), (self.config.packet_serializer)(packet)), addr.clone()));
                true
            },
            None => false
        }
    }

    /**
     * Send a packet to multiple addresses
     */
    pub fn send_to_many(&mut self, packet: &T, addrs: &Vec<SocketAddr>) {
        for addr in addrs.iter() {
            self.send_to(packet, addr);
        }
    }

    /**
     * Send a packet to every connected client
     */
    pub fn send_to_all(&mut self, packet: &T) {
        for addr in self.connections.values() {
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
        self.reader_send.send(TaskCommand::Disconnect);
    }
}
