use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;
use std::io::{IoResult, IoError, OtherIoError, TimedOut};
use std::io::Timer;
use std::comm::{Disconnected, Empty, Select};
use std::time::duration::Duration;
use packet::{Packet, PacketConnect, PacketAccept, PacketReject, Command, Disconnect};


pub enum ConnectionState {
    CommsDisconnected,
    CommsConnecting,
    CommsConnected
}

fn reader_process(mut reader: UdpSocket, reader_sub_out: Sender<Packet>, reader_sub_in: Receiver<Command>, target_addr: SocketAddr, protocol_id: u32) {
    let mut buf = [0, ..255];
    reader.set_timeout(Some(1000));
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

fn writer_process(mut writer: UdpSocket, writer_sub_out: Sender<Command>, writer_sub_in: Receiver<Packet>, target_addr: SocketAddr) {
    for msg in writer_sub_in.iter() {
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
                    connection_state: CommsDisconnected
                };

                if client.connection_dance() {
                    Ok(client)
                } else {
                    Err(IoError {
                        kind: OtherIoError,
                        desc: "Failed to connect",
                        detail: None
                    })
                }
            }
            Err(e) => Err(e)
        }
    }

    /**
     * A blocking connection request
     */
    fn connection_dance(&mut self) -> bool {
        self.connection_state = CommsConnecting;
        let mut timer = Timer::new().unwrap();
        let mut attempts = 0u;

        while attempts < 3 && match self.connection_state { CommsConnecting => true, _ => false } {

            self.writer_send.send(Packet::connect(self.protocol_id));

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
                            self.connection_state = CommsConnected;
                        }
                        PacketReject => {
                            self.connection_state = CommsDisconnected;
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
            CommsConnecting => {
                self.connection_state = CommsDisconnected;
                false
            },
            CommsDisconnected => false,
            CommsConnected => {
                true
            }
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

impl Drop for Client {

    fn drop(&mut self) {
        self.reader_send.send(Disconnect);
    }
}
