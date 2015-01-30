use std::old_io::{IoResult, IoError, OtherIoError};
use std::old_io::BufReader;
use std::num::FromPrimitive;

///Headers for various different built-in message types
#[derive(FromPrimitive, Clone, Show, PartialEq, Copy)]
pub enum PacketType {
    Connect = 0,
    Accept,
    Reject,
    Disconnect,
    Message
}

///The underlying shape for transferring data.
#[derive(Clone)]
pub struct Packet {
    pub protocol_id: u32,
    ///The current id of the packet
    pub sequence_id: u16,
    pub packet_type: PacketType,
    ///Serialized user data goes in here
    pub packet_content: Option<Vec<u8>>
}

///Commands to send to subprocesses
pub enum TaskCommand {
    Disconnect,
}

impl Packet {

    pub fn connect(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketType::Connect,
            packet_content: None
        }
    }

    pub fn disconnect(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketType::Disconnect,
            packet_content: None
        }
    }
    
    pub fn accept(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketType::Accept,
            packet_content: None
        }
    }

    pub fn reject(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketType::Reject,
            packet_content: None
        }
    }

    pub fn message(protocol_id: u32, sequence_id: u16, message: Vec<u8>) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketType::Message,
            packet_content: Some(message)
        }
    }

    pub fn deserialize(raw: &[u8]) -> IoResult<Packet> {
        let mut r = BufReader::new(raw);
        let protocol_id = try!(r.read_be_u32());
        let sequence_id = try!(r.read_be_u16());
        let packet_type = try!(r.read_byte());
        let content = try!(r.read_to_end());

        match FromPrimitive::from_u8(packet_type) {
            Some(packet_type) => {
                Ok(Packet {
                    protocol_id: protocol_id,
                    sequence_id: sequence_id,
                    packet_type: packet_type,
                    packet_content: if content.len() > 0 { Some(content) } else { None }
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
        let mut w = vec![];
        try!(w.write_be_u32(self.protocol_id));
        try!(w.write_be_u16(self.sequence_id));
        try!(w.write_u8(self.packet_type as u8));
        match self.packet_content {
            Some(ref content) => {
                try!(w.write(content.as_slice()))
            },
            None => ()
        }
        Ok(w)
    }
}
