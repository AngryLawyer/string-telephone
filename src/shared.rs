use std::time::duration::Duration;
use std::u16;
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

/**
 * A helper struct to maintain packet ordering and acks
 */
#[derive(Clone)]
pub struct SequenceManager {
    pub last_sent_sequence_id: u16,
    pub last_received_sequence_id: u16
}

impl SequenceManager {
    /**
     * Create a new SequenceManager
     */
    pub fn new() -> SequenceManager {
        SequenceManager {
            last_sent_sequence_id: 0,
            last_received_sequence_id: 0
        }
    }

    /**
     * Generate a new sequence ID for us
     */
    pub fn next_sequence_id(&mut self) -> u16 {
        self.last_sent_sequence_id += 1;
        self.last_sent_sequence_id
    }

    /**
     * Is a packet classed as newer than the last we received?
     */
    pub fn packet_is_newer(&self, sequence_id: u16) -> bool {
        let max = u16::MAX;
        return (sequence_id > self.last_received_sequence_id) && (sequence_id - self.last_received_sequence_id <= max/2) ||
            (self.last_received_sequence_id > sequence_id) && (self.last_received_sequence_id - sequence_id > max/2);
    }

    /**
     * Set the last packet we received
     */
    pub fn set_newest_packet(&mut self, sequence_id: u16) {
        self.last_received_sequence_id = sequence_id;
    }
}
