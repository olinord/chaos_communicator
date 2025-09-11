use crate::communicator::ChaosCommunicationError::{CouldNotSendMessage, NoSenderFound};
use crate::message::ChaosMessage;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct ChaosReceiver {
    receiver: Receiver<ChaosMessage>,
}

impl ChaosReceiver {
    pub fn receive(&mut self) -> Option<ChaosMessage> {
        if self.receiver.is_empty() {
            println!("Receiver is empty");
            return None;
        }
        let message = self.receiver.recv();
        if let Ok(message) = message {
            println!("Message received");
            return Some(message);
        }
        println!("Failed to receive message");
        return None;
    }
}

#[derive(Clone)]
pub struct ChaosCommunicator {
    senders_and_receivers: HashMap<u64, Vec<(Sender<ChaosMessage>, Receiver<ChaosMessage>)>>,
}

impl ChaosCommunicator {
    pub fn new() -> Self {
        ChaosCommunicator {
            senders_and_receivers: HashMap::new(),
        }
    }

    pub fn register_for<T: Hash>(&mut self, event: T) -> ChaosReceiver {
        let mut hasher = DefaultHasher::new();
        event.hash(&mut hasher);
        let hash_value = hasher.finish();

        let (sender, receiver) = unbounded::<ChaosMessage>();
        let receiver_clone = receiver.clone();
        match self.senders_and_receivers.get_mut(&hash_value) {
            Some(vec) => {
                vec.push((sender, receiver));
            }
            None => {
                self.senders_and_receivers
                    .insert(hash_value, vec![(sender, receiver)]);
            }
        }

        return ChaosReceiver {
            receiver: receiver_clone,
        };
    }

    pub fn send_message(&self, message: ChaosMessage) -> Result<(), ChaosCommunicationError> {
        // find the channel to post on
        let event = message.get_event();
        match self.senders_and_receivers.get(&message.get_event()) {
            Some(senders_and_receivers) => {
                for (sender, _receiver) in senders_and_receivers {
                    let result = sender.send(message.clone());
                    if result.is_err() {
                        // Could not send the error for some reason
                        let message = format!(
                            "Message could not be sent for event {}. Likely due to channel being closed.",
                            event
                        );
                        return Err(CouldNotSendMessage(message));
                    }
                }
                return Ok(());
            }
            None => {
                return Err(NoSenderFound(format!(
                    "No sender found for event {}",
                    event
                )));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChaosCommunicationError {
    CouldNotSendMessage(String),
    NoSenderFound(String),
}

impl Display for ChaosCommunicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChaosCommunicationError::CouldNotSendMessage(msg) => {
                write!(f, "Could not send message: {}", msg)
            }
            ChaosCommunicationError::NoSenderFound(msg) => write!(f, "No sender found: {}", msg),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ChaosMessageBuilder;
    use std::fmt;
    use std::thread;

    #[test]
    fn message_can_be_sent_to_listeners() {
        let mut communicator = ChaosCommunicator::new();
        let mut r = communicator.register_for(987654321);

        thread::spawn(move || {
            let message = ChaosMessageBuilder::new()
                .with_param("test", 1123)
                .build_for_event(987654321);
            assert_eq!(communicator.send_message(message).is_ok(), true);
        })
        .join()
        .unwrap();

        assert_eq!(r.receive().unwrap().get("test"), Some(1123));
    }

    #[test]
    fn if_no_message_is_available_then_no_message_is_received() {
        let mut communicator = ChaosCommunicator::new();
        let mut r = communicator.register_for(TestEnumEvent::Event1);

        assert_eq!(r.receive().is_none(), true);
    }

    #[derive(Hash)]
    enum TestEnumEvent {
        Event1 = 1,
    }

    impl fmt::Display for TestEnumEvent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self)
        }
    }

    #[test]
    fn message_can_be_registered_for_enum() {
        let mut communicator = ChaosCommunicator::new();
        let mut r = communicator.register_for(TestEnumEvent::Event1);

        thread::spawn(move || {
            let message = ChaosMessageBuilder::new()
                .with_param("some_parameter", 1234)
                .build_for_event(TestEnumEvent::Event1);
            assert_eq!(communicator.send_message(message).is_ok(), true);
        })
        .join()
        .unwrap();

        assert_eq!(r.receive().unwrap().get("some_parameter"), Some(1234));
    }

    #[test]
    fn multiple_receivers_can_be_registered_for_the_same_event() {
        let mut communicator = ChaosCommunicator::new();
        let mut r1 = communicator.register_for(TestEnumEvent::Event1);
        let mut r2 = communicator.register_for(TestEnumEvent::Event1);

        thread::spawn(move || {
            let message = ChaosMessageBuilder::new()
                .with_param("some_parameter", 1234)
                .build_for_event(TestEnumEvent::Event1);
            assert_eq!(communicator.send_message(message).is_ok(), true);
        })
        .join()
        .unwrap();

        assert_eq!(r1.receive().unwrap().get("some_parameter"), Some(1234));
        assert_eq!(r2.receive().unwrap().get("some_parameter"), Some(1234));
    }

    #[test]
    fn sending_event_that_hasnt_been_registered_returns_error() {
        let communicator = ChaosCommunicator::new();
        let result =
            communicator.send_message(ChaosMessageBuilder::new().build_for_event("I dont exist"));

        assert_eq!(result.is_err(), true);
    }
}
