use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;
use std::io::{IoResult, IoError, OtherIoError, TimedOut};
use std::io::Timer;
use std::comm::{Disconnected, Empty, Select};
use std::time::duration::Duration;
use packet::{Packet, PacketConnect, PacketAccept, PacketReject, Command, Disconnect};


/**
 * What we want:
 * Server sits and loops about. Has a send/receive buffer
 * Has a broadcast method?
 * Has a send method
 * Has a read method, to pump items out of the receive buffer
 * Has a read iterator?
 */

struct ServerManager {
    pub addr: SocketAddr,

    protocol_id: u32,
    reader_send: Sender<Command>,
    reader_receive: Receiver<(Packet, SocketAddr)>,
}

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
                            Err(Empty) => {
                                //Keep going
                            }
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}

impl ServerManager {
    pub fn new(protocol_id: u32, addr: SocketAddr) -> IoResult<ServerManager> {
        match UdpSocket::bind(addr) {
            Ok(reader) => {
                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                spawn(proc() {
                    reader_process(reader, reader_sub_out, reader_sub_in, protocol_id);
                });
                
                Ok(ServerManager {
                    protocol_id: protocol_id,
                    addr: addr,
                    reader_send: reader_out,
                    reader_receive: reader_in
                })
            }
            Err(e) => Err(e)
        }
    }
}

impl Drop for ServerManager {

    fn drop(&mut self) {
        self.reader_send.send(Disconnect);
    }
}
