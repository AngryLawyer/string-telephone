pub struct ConnectionConfig<T> {
    pub protocol_id: u32,
    pub timeout_period: u32,
    pub packet_deserializer: fn(&Vec<u8>) -> T,
    pub packet_serializer: fn(&T) -> Vec<u8>
}
