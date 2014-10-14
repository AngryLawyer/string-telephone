use std::time::duration::Duration;
/**
 * General configuration for a connection
 */
pub struct ConnectionConfig<T> {
    /// A shared ID to identify whether a connection should be accepted
    pub protocol_id: u32,
    /// How long we should wait before hanging up
    pub timeout_period: Duration,
    /// A function to turn raw data into our packet format
    pub packet_deserializer: fn(&Vec<u8>) -> Option<T>,
    /// A function to turn a packet into raw data
    pub packet_serializer: fn(&T) -> Vec<u8>
}

impl <T> ConnectionConfig <T> {

    /**
     * Create a new ConnectionConfig object
     */
    pub fn new(protocol_id: u32, timeout_period: Duration, packet_deserializer: fn(&Vec<u8>) -> Option<T>, packet_serializer: fn(&T) -> Vec<u8>) -> ConnectionConfig<T> {
        ConnectionConfig {
            protocol_id: protocol_id,
            timeout_period: timeout_period,
            packet_deserializer: packet_deserializer,
            packet_serializer: packet_serializer
        }
    }
}
