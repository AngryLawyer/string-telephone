use std::io::{IoResult, IoError, OtherIoError};
use std::io::{BufReader, MemWriter};

///Headers for various different built-in message types
#[deriving(FromPrimitive, Clone, Show, PartialEq)]
pub enum PacketType {
    PacketConnect = 0,
    PacketAccept,
    PacketReject,
    PacketDisconnect,
    PacketMessage
}

///The underlying shape for transferring data.
#[deriving(Clone)]
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
            packet_type: PacketConnect,
            packet_content: None
        }
    }

    pub fn disconnect(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketDisconnect,
            packet_content: None
        }
    }
    
    pub fn accept(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketAccept,
            packet_content: None
        }
    }

    pub fn reject(protocol_id: u32, sequence_id: u16) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketReject,
            packet_content: None
        }
    }

    pub fn message(protocol_id: u32, sequence_id: u16, message: Vec<u8>) -> Packet {
        Packet {
            protocol_id: protocol_id,
            sequence_id: sequence_id,
            packet_type: PacketMessage,
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
        let mut w = MemWriter::new();
        try!(w.write_be_u32(self.protocol_id));
        try!(w.write_be_u16(self.sequence_id));
        try!(w.write_u8(self.packet_type as u8));
        match self.packet_content {
            Some(ref content) => {
                try!(w.write(content.as_slice()))
            },
            None => ()
        }
        Ok(w.unwrap())
    }
}
