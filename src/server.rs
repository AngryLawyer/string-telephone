/**
 * What we want:
 * Server sits and loops about. Has a send/receive buffer
 * Has a broadcast method?
 * Has a send method
 * Has a read method, to pump items out of the receive buffer
 * Has a read iterator?
 */

struct ServerManager<T> {
    receiver: (Sender<T>, Receiver<T>)
}

impl<T> ServerManager<T> {
    pub fn new() -> ServerManager<T> {
        ServerManager {
            receiver: channel()
        }
    }
}

fn main () {
}
