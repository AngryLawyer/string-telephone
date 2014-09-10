use std::io::{IoResult, IoError, OtherIoError};
use std::io::{BufReader, MemWriter};

#[deriving(FromPrimitive)]
pub enum PacketType {
    PacketConnect = 0,
    PacketAccept,
    PacketReject,
    PacketDisconnect,
    PacketMessage
}

pub struct Packet {
    pub protocol_id: u32,
    pub packet_type: PacketType,
    pub packet_content: Vec<u8>
}

pub enum Command {
    Disconnect,
}

impl Packet {
    pub fn deserialize(raw: &[u8]) -> IoResult<Packet> {
        let mut r = BufReader::new(raw);
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
            None => Err(IoError {
                kind: OtherIoError,
                desc: "Invalid packet type",
                detail: None
            })
        }

    }

    pub fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut w = MemWriter::new();
        try!(w.write_be_u32(self.protocol_id));
        try!(w.write_u8(self.packet_type as u8));
        try!(w.write(self.packet_content.as_slice()));
        Ok(w.unwrap())
    }
}
