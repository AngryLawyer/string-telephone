extern crate serialize;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::MemReader;
use std::io::{IoResult, IoError, IoErrorKind, OtherIoError};
use std::comm::TryRecvError;

#[deriving(FromPrimitive)]
enum PacketType {
    PacketConnect = 0,
    PacketDisconnect,
    PacketMessage
}

struct Packet {
    protocol_id: u32,
    packet_type: PacketType,
    packet_content: Vec<u8>
}

enum Command {
    Disconnect
}

impl Packet {
    fn deserialize(raw: &[u8]) -> IoResult<Packet> {
        let mut r = MemReader::new(Vec::from_slice(raw));
        let protocol_id = try!(r.read_be_u32());
        let packet_type = try!(r.read_byte());
        let content = try!(r.read_to_end());

        match FromPrimitive::from_u8(packet_type) {
            Some(packet_type) => {
                Ok(Packet {
                    protocol_id: protocol_id,
                    packet_type: packet_type,
                    packet_content: content
                })
            },
            None => Err(IoError{
                kind: OtherIoError,
                desc: "Invalid packet type",
                detail: None
            })
        }

    }

    fn serialize(&self) -> Vec<u8> {
        vec![]
    }
}

struct Connection {
    addr: SocketAddr,
    reader_comms: Option<(Sender<Command>, Receiver<Packet>)>,
    writer_comms: Option<(Sender<Packet>, Receiver<Command>)>
}

fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<Packet>, reader_sub_in: Receiver<Command>) {
    let mut buf = [0, ..255];
    loop {
        match reader.recv_from(buf) {
            Ok((amt, src)) => {
                match Packet::deserialize(buf) {
                    Ok(packet) => {
                        //TODO: Validate this packet is actually for us
                        //Packet type, sender
                        reader_sub_out.send(packet);
                    },
                    Err(_) => ()
                }
            }
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }
    }
}

fn writer_process(mut writer: UdpSocket, writer_sub_out: Sender<Command>, writer_sub_in: Receiver<Packet>, target_addr: SocketAddr) {
    for msg in writer_sub_in.iter() {
        writer.send_to(msg.serialize().as_slice(), target_addr);
    }
}

impl Connection {
    pub fn new(addr: SocketAddr) -> Connection {
        Connection {
            addr: addr,
            reader_comms: None,
            writer_comms: None
        }
    }

    pub fn connect(&mut self, target_addr: SocketAddr) {
         match UdpSocket::bind(self.addr) {
            Ok(reader) => {
                let writer = reader.clone();

                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                spawn(proc() {
                    reader_process(reader, reader_sub_out, reader_sub_in);
                });

                let (writer_out, writer_sub_in) = channel();
                let (writer_sub_out, writer_in) = channel();
                spawn(proc() {
                    writer_process(writer, writer_sub_out, writer_sub_in, target_addr);
                });

                self.reader_comms = Some((reader_out, reader_in));
                self.writer_comms = Some((writer_out, writer_in));
            }
            Err(e) => fail!("couldn't bind socket: {}", e)
        };
    }

    pub fn poll(&mut self) -> Option<Packet> {
        match self.reader_comms {
            Some((_, ref mut reader_in)) => {
                match reader_in.try_recv() {
                    Ok(value) => Some(value),
                    _ => None
                }
            },
            None => None
        }
    }
}

fn main () {
    Connection::new(SocketAddr{ip: Ipv4Addr(127, 0, 0, 1), port: 0});
}
