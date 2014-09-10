use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::{IoResult, IoError, IoErrorKind, OtherIoError};
use std::io::Timer;
use std::comm::TryRecvError;
use std::time::duration::Duration;
use std::comm::Select;
use packet::{Packet, Command, PacketConnect, PacketAccept, PacketReject};


fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<Packet>, reader_sub_in: Receiver<Command>, target_addr: SocketAddr, protocol_id: u32) {
    let mut buf = [0, ..255];
    loop {
        match reader.recv_from(buf) {
            Ok((amt, src)) => {
                if src == target_addr {
                    match Packet::deserialize(buf.slice_to(amt)) {
                        Ok(packet) => {
                            if packet.protocol_id == protocol_id {
                                reader_sub_out.send(packet);
                            }
                        },
                        Err(_) => ()
                    }
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

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected
}

/**
 * Clientside implementation of UDP networking
 */
pub struct Client {
    pub addr: SocketAddr,
    pub target_addr: SocketAddr,

    protocol_id: u32,
    connection_state: ConnectionState,

    reader_send: Sender<Command>,
    reader_receive: Receiver<Packet>,
    writer_send: Sender<Packet>,
    writer_receive: Receiver<Command>
}


impl Client {
    /**
     * Connect our Client to a target Server
     */
    pub fn connect(addr: SocketAddr, target_addr: SocketAddr, protocol_id: u32) -> IoResult<Client> {
         match UdpSocket::bind(addr) {
            Ok(reader) => {
                let writer = reader.clone();

                let (reader_out, reader_sub_in) = channel();
                let (reader_sub_out, reader_in) = channel();

                spawn(proc() {
                    reader_process(reader, reader_sub_out, reader_sub_in, target_addr, protocol_id);
                });

                let (writer_out, writer_sub_in) = channel();
                let (writer_sub_out, writer_in) = channel();

                spawn(proc() {
                    writer_process(writer, writer_sub_out, writer_sub_in, target_addr);
                });

                let mut client = Client {
                    addr: addr,
                    target_addr: target_addr,
                    reader_send: reader_out,
                    reader_receive: reader_in,
                    writer_send: writer_out,
                    writer_receive: writer_in,
                    protocol_id: protocol_id,
                    connection_state: Disconnected
                };

                client.connection_dance();
                match client.connection_state {
                    Connected => {
                        Ok(client)
                    },
                    _ => {
                        Err(IoError{
                            kind: OtherIoError,
                            desc: "Failed to connect",
                            detail: None
                        })
                    }
                }
            }
            Err(e) => Err(e)
        }
    }

    /**
     * A blocking connection request
     */
    fn connection_dance(&mut self) {
        self.connection_state = Connecting;
        let mut timer = Timer::new().unwrap();
        let mut attempts = 0u;

        while attempts < 5 && match self.connection_state { Connecting => true, _ => false } {

            self.writer_send.send(Packet {
                protocol_id: self.protocol_id,
                packet_type: PacketConnect,
                packet_content: vec![]
            });

            let timeout = timer.oneshot(Duration::seconds(5));
            //FIXME: Replace with the select! macro when it starts working
            let sel = Select::new();
            let mut reader = sel.handle(&self.reader_receive);
            let mut timeout = sel.handle(&timeout);
            unsafe { reader.add(); timeout.add(); }
            let ret = sel.wait();
            if ret == reader.id() {
                let packet = self.reader_receive.recv();
                {
                    match packet.packet_type {
                        PacketAccept => {
                            self.connection_state = Connected;
                        }
                        PacketReject => {
                            self.connection_state = Disconnected;
                        }
                        _ => (),
                    }
                }
            } else if ret == timeout.id() {
                let () = timeout.recv();
                { attempts += 1; }
            } else {
                fail!("What");
            }
        }

        //We didn't manage to connect
        match self.connection_state {
            Connecting => {
                self.connection_state = Disconnected
            },
            _ => ()
        }
    }

    /**
     * Pop the last event off of our comms queue, if any
     */
    pub fn poll(&mut self) -> Option<Packet> {
        match self.reader_receive.try_recv() {
            Ok(value) => Some(value),
            _ => None
        }
    }
}
