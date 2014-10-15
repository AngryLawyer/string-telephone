pub fn deserializer(message: &Vec<u8>) -> Option<Vec<u8>> {
    Some(message.clone())
}

pub fn serializer(packet: &Vec<u8>) -> Vec<u8> {
    packet.clone()
}

