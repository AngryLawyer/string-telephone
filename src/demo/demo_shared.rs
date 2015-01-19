use core::borrow::Cow;

pub fn deserializer(message: &Vec<u8>) -> Option<String> {
    match String::from_utf8_lossy(message.as_slice()) {
        Cow::Borrowed(slice) => Some(slice.to_string()),
        Cow::Owned(item) => Some(item)
    }
}

pub fn serializer(packet: &String) -> Vec<u8> {
    packet.clone().into_bytes()
}
