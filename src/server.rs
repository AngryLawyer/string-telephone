/**
 * What we want:
 * Server sits and loops about. Has a send/receive buffer
 * Has a broadcast method?
 * Has a send method
 * Has a read method, to pump items out of the receive buffer
 * Has a read iterator?
 */
use packet::{Packet, Command};

struct ServerManager {
    pub addr: SocketAddr,

    protocol_id: u32,
    reader_send: Sender<Command>,
    reader_receive: Receiver<Packet>,
}

impl ServerManager {
    pub fn new(protocol_id: u32) -> ServerManager {
        
        ServerManager {protocol_id: protocol_id}
    }
}
