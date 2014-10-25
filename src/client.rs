use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;
use std::io::{IoResult, IoError, OtherIoError, TimedOut};
use std::io::Timer;
use std::comm::{Disconnected, Empty, Select};
use std::time::duration::Duration;
use packet::{Packet, PacketAccept, PacketReject, PacketDisconnect, PacketMessage, TaskCommand, Disconnect};
use shared::{ConnectionConfig, SequenceManager};
use time::now;


/**
 * The current state of a connection
 */
pub enum ConnectionState {
    CommsDisconnected,
    CommsConnecting,
    CommsConnected
}

/**
 * A Poll attempt failed for some reason
 */
#[deriving(Show)]
pub enum PollFailResult {
    PollEmpty,
    PollDisconnected
}

fn reader_process(mut reader: UdpSocket, send: Sender<Packet>, recv: Receiver<TaskCommand>, target_addr: SocketAddr, protocol_id: u32, timeout_period: Duration) {
    let mut buf = [0, ..1023];
    reader.set_timeout(Some(1000));

    let mut expires = now().to_timespec().sec + timeout_period.num_seconds();

    loop {
        match reader.recv_from(buf) {
            Ok((amt, src)) => {
                if src == target_addr {
                    match Packet::deserialize(buf.slice_to(amt)) {
                        Ok(packet) => {
                            if packet.protocol_id == protocol_id {
                                match send.send_opt(packet) {
                                    Ok(()) => {
                                        expires = now().to_timespec().sec + timeout_period.num_seconds();
                                    },
                                    Err(_) => {
                                        //Other end hung up, we should give up
                                        break;
                                    }
                                }
                            }
                        },
                        Err(_) => ()
                    }
                }
            },
            Err(e) => {
                match e.kind {
                    TimedOut => {
                        match recv.try_recv() {
                            Ok(Disconnect) => {
                                break;
                            },
                            Err(Disconnected) => {
                                break;
                            },
                            Err(Empty) => {
                                //Keep going
                            }
                        }
                    },
                    _ => ()
                }
            }
        };
        if now().to_timespec().sec > expires {
            //FIXME: Need a nicer way of ignoring failure for this
            //FIXME: Bad sequence ID!
            match send.send_opt(Packet::disconnect(protocol_id, 0)) {
                _ => break
            }
        }
    }
}

fn writer_process(mut writer: UdpSocket, recv: Receiver<Packet>, target_addr: SocketAddr) {
    for msg in recv.iter() {
        match msg.serialize() {
            Ok(msg) => {
                match writer.send_to(msg.as_slice(), target_addr) {
                    Ok(()) => (),
                    Err(e) => println!("Error sending data - {}", e)
                }
            },
            Err(_) => ()
        }
    }
}

/**
 * Clientside implementation of UDP networking
 */
pub struct Client <T> {
    ///The socket we should use locally
    pub addr: SocketAddr,
    ///The socket of the server we intent to connect to
    pub target_addr: SocketAddr,
    ///Basic configuration for connecting
    pub config: ConnectionConfig<T>,

    ///What's the current state of our connection
    pub connection_state: ConnectionState,

    reader_send: Sender<TaskCommand>,
    reader_receive: Receiver<Packet>,
    writer_send: Sender<Packet>,

    sequence_manager: SequenceManager
}

/**
 * Additional configuration options for a Client connection
 */
pub struct ClientConnectionConfig {
    ///How many times should we ask for a connection before giving up?
    pub max_connect_retries: uint,
    ///How long should each connection request await an answer?
    pub connect_attempt_timeout: Duration
}

impl ClientConnectionConfig {

    /**
     * Create a new ClientConnectionConfig object
     */
    pub fn new(max_connect_retries: uint, connect_attempt_timeout: Duration) -> ClientConnectionConfig {
        ClientConnectionConfig {
            max_connect_retries: max_connect_retries,
            connect_attempt_timeout: connect_attempt_timeout
        }
    }
}

impl <T> Client <T> {

    /**
     * Connect our Client to a target Server.
     * Will block until either a valid connection is made, or we give up
     */
    pub fn connect(addr: SocketAddr, target_addr: SocketAddr, config: ConnectionConfig<T>, client_connection_config: ClientConnectionConfig) -> IoResult<Client<T>> {
         match UdpSocket::bind(addr) {
            Ok(reader) => {
                let writer = reader.clone();

                let (reader_send, reader_task_receive) = channel();
                let (reader_task_send, reader_receive) = channel();

                let protocol_id = config.protocol_id;
                let timeout_period = config.timeout_period;

                spawn(proc() {
                    reader_process(reader, reader_task_send, reader_task_receive, target_addr, protocol_id, timeout_period);
                });

                let (writer_send, writer_task_receive) = channel();

                spawn(proc() {
                    writer_process(writer, writer_task_receive, target_addr);
                });

                let mut client = Client {
                    addr: addr,
                    target_addr: target_addr,
                    reader_send: reader_send,
                    reader_receive: reader_receive,
                    writer_send: writer_send,
                    connection_state: CommsDisconnected,
                    config: config,
                    sequence_manager: SequenceManager::new()
                };

                if client.connection_dance(client_connection_config.max_connect_retries, client_connection_config.connect_attempt_timeout) {
                    Ok(client)
                } else {
                    Err(IoError {
                        kind: OtherIoError,
                        desc: "Failed to connect",
                        detail: None
                    })
                }
            }
            Err(e) => Err(e)
        }
    }

    /**
     * A blocking connection request
     */
    fn connection_dance(&mut self, max_attempts: uint, timeout: Duration) -> bool {
        self.connection_state = CommsConnecting;
        let mut timer = Timer::new().unwrap();
        let mut attempts = 0u;

        while attempts < max_attempts && match self.connection_state { CommsConnecting => true, _ => false } {
            self.writer_send.send(Packet::connect(self.config.protocol_id, self.sequence_manager.next_sequence_id()));

            let timeout = timer.oneshot(timeout);

            //FIXME: Replace with the select! macro when it starts working
            let sel = Select::new();
            let mut reader = sel.handle(&self.reader_receive);
            let mut timeout = sel.handle(&timeout);
            unsafe { reader.add(); timeout.add(); }
            let ret = sel.wait();
            if ret == reader.id() {
                let packet = self.reader_receive.recv();
                match packet.packet_type {
                    PacketAccept => {
                        self.connection_state = CommsConnected;
                    }
                    PacketReject => {
                        self.connection_state = CommsDisconnected;
                    }
                    PacketDisconnect => {
                        self.connection_state = CommsDisconnected;
                    }
                    _ => (),
                }
            } else if ret == timeout.id() {
                let () = timeout.recv();
                attempts += 1;
            } else {
                unreachable!();
            }
        }

        match self.connection_state {
            CommsConnecting => {
                self.connection_state = CommsDisconnected;
                false
            },
            CommsDisconnected => false,
            CommsConnected => {
                true
            }
        }
    }

    /**
     * Pop the last event off of our comms queue, if any
     */
    pub fn poll(&mut self) -> Result<T, PollFailResult> {
        match self.connection_state {
            CommsConnected => {
                let mut result = Err(PollEmpty);
                loop {
                    match self.reader_receive.try_recv() {
                        Ok(value) => {
                            match value.packet_type {
                                PacketDisconnect => {
                                    self.connection_state = CommsDisconnected;
                                    result = Err(PollDisconnected);
                                    break;
                                },
                                PacketMessage => {
                                    //Are we expecting this packet?
                                    if self.sequence_manager.packet_is_newer(value.sequence_id) {
                                        self.sequence_manager.set_newest_packet(value.sequence_id);
                                        match (self.config.packet_deserializer)(&value.packet_content.unwrap()) {
                                            Some(deserialized) => {
                                                result = Ok(deserialized);
                                                break;
                                            },
                                            None => ()
                                        }
                                    }
                                },
                                _ => ()
                            }
                        },
                        _ => break
                    };
                }
                result
            },
            _ => Err(PollDisconnected)
        }
    }

    /**
     * Send a packet to the server
     */
    pub fn send(&mut self, packet: &T) {
        match self.writer_send.send_opt(Packet::message(self.config.protocol_id, self.sequence_manager.next_sequence_id(), (self.config.packet_serializer)(packet))) {
            _ => () //FIXME: We shouldn't discard errors here
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Client<T> {

    fn drop(&mut self) {
        match (self.reader_send.send_opt(Disconnect),  self.writer_send.send_opt(Packet::disconnect(self.config.protocol_id, self.sequence_manager.next_sequence_id()))) {
            _ => () //FIXME: This is a bad way of discarding errors
        }
    }
}
