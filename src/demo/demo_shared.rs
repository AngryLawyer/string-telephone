use collections::str::{Slice, Owned};

pub fn deserializer(message: &Vec<u8>) -> String {
    match String::from_utf8_lossy(message.as_slice()) {
        Slice(slice) => slice.to_string(),
        Owned(item) => item
    }
}

pub fn serializer(packet: &String) -> Vec<u8> {
    packet.clone().into_bytes()
}
