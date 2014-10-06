/**
 * General configuration for a connection
 */
pub struct ConnectionConfig<T> {
    pub protocol_id: u32,
    pub timeout_period: u32,
    pub packet_deserializer: fn(&Vec<u8>) -> Option<T>,
    pub packet_serializer: fn(&T) -> Vec<u8>
}

impl <T> ConnectionConfig <T> {

    pub fn new(protocol_id: u32, timeout_period: u32, packet_deserializer: fn(&Vec<u8>) -> Option<T>, packet_serializer: fn(&T) -> Vec<u8>) -> ConnectionConfig<T> {
        ConnectionConfig {
            protocol_id: protocol_id,
            timeout_period: timeout_period,
            packet_deserializer: packet_deserializer,
            packet_serializer: packet_serializer
        }
    }
}
