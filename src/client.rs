use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::{IoResult, IoError, IoErrorKind, OtherIoError};
use std::comm::TryRecvError;
use packet::{Packet, Command};


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

/**
 * Clientside implementation of UDP networking
 */
struct Client {
    pub addr: SocketAddr,
    pub target_addr: SocketAddr,

    reader_send: Sender<Command>,
    reader_receive: Receiver<Packet>,
    writer_send: Sender<Packet>,
    writer_receive: Receiver<Command>
}


impl Client {

    /**
     * Connect our Client to a target Server
     */
    pub fn connect(addr: SocketAddr, target_addr: SocketAddr) -> IoResult<Client> {
         match UdpSocket::bind(addr) {
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

                Ok(Client {
                    addr: addr,
                    target_addr: target_addr,
                    reader_send: reader_out,
                    reader_receive: reader_in,
                    writer_send: writer_out,
                    writer_receive: writer_in
                })
            }
            Err(e) => Err(e)
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

fn main () {
    Client::connect(SocketAddr{ip: Ipv4Addr(127, 0, 0, 1), port: 0}, SocketAddr{ip: Ipv4Addr(127, 0, 0, 1), port: 23232});
}
