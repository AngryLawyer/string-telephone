use std::io::net::ip::SocketAddr;
use std::io::net::udp::UdpSocket;

pub fn get_message(socket: &mut UdpSocket) -> (Vec<u8>, SocketAddr) {
    let mut buf = [0, ..255];
    match socket.recv_from(buf) {
        Ok((amt, src)) => (buf.slice_to(amt).to_vec(), src),
        Err(e) => fail!("Socket didn't get a message - {}", e)
    }
}

pub fn deserializer(message: &Vec<u8>) -> Option<Vec<u8>> {
    Some(message.clone())
}

pub fn serializer(packet: &Vec<u8>) -> Vec<u8> {
    packet.clone()
}
