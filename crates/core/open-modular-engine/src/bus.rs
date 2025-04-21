use fancy_constructor::new;
use rtrb::{
    Consumer,
    Producer,
    PushError,
    RingBuffer,
};

use crate::protocol::Protocol;

// =================================================================================================
// Bus
// =================================================================================================

#[derive(Debug, Default)]
pub struct Bus {}

impl Bus {
    #[must_use]
    pub fn split(self) -> (BusSender, BusReceiver) {
        let buffer = RingBuffer::new(32);
        let sender = BusSender::new(buffer.0);
        let receiver = BusReceiver::new(buffer.1);

        (sender, receiver)
    }
}

// -------------------------------------------------------------------------------------------------

// Receiver

#[derive(new, Debug)]
#[new(vis())]
pub struct BusReceiver {
    consumer: Consumer<Protocol>,
}

impl BusReceiver {
    pub fn receive(&mut self) -> Option<Protocol> {
        self.consumer.pop().ok()
    }
}

// -------------------------------------------------------------------------------------------------

// Sender

#[derive(new, Debug)]
#[new(vis())]
pub struct BusSender {
    producer: Producer<Protocol>,
}

impl BusSender {
    pub fn send(&mut self, protocol: Protocol) -> Option<Protocol> {
        self.producer
            .push(protocol)
            .map_or_else(|PushError::Full(protocol)| Some(protocol), |()| None)
    }
}
