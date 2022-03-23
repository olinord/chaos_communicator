use std::any::{Any};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use crossbeam_channel::{Sender, Receiver, unbounded};
use crate::communicator::ChaosCommunicationError::{CouldNotSendMessage, NoSenderFound};
use crate::message::ChaosMessage;

pub struct ChaosCommunicator {
    senders_and_receivers: HashMap<u64, (Sender<ChaosMessage>, Receiver<ChaosMessage>)>
}

pub enum ChaosCommunicationError {
    CouldNotSendMessage(String),
    NoSenderFound(String)
}

impl ChaosCommunicator {
    pub fn new() -> Self {
        ChaosCommunicator{
            senders_and_receivers: HashMap::new()
        }
    }

    pub fn register_for<T: Any + Hash + Display>(&mut self, event: T) -> Receiver<ChaosMessage>{
        let mut hasher = DefaultHasher::new();
        event.hash(&mut hasher );
        let hash_value = hasher.finish();

        match self.senders_and_receivers.get(&hash_value){
            Some((_, receiver)) => {
                return receiver.clone();
            },
            None => {
                let (sender, receiver) = unbounded::<ChaosMessage>();
                let ret = receiver.clone();
                self.senders_and_receivers.insert( hash_value, (sender, receiver));
                return ret;
            }
        }
    }

    pub fn send_message<T: Any + Hash + Display + Copy>(&self, event: T, message: ChaosMessage) -> Result<(), ChaosCommunicationError> {
        let mut hasher = DefaultHasher::new();
        event.hash(&mut hasher );
        let hash_value = hasher.finish();

        // find the channel to post on
        match self.senders_and_receivers.get(&hash_value)
        {
            Some((sender, _)) => {
                let result = sender.send(message);
                if result.is_err() {
                    // Could not send the error for some reason
                    let message = format!(
                        "Message could not be sent for event {}. Likely due to channel being closed.", event);
                    return Err(CouldNotSendMessage(message))
                }
                return Ok(());
            },
            None => {
                return Err(NoSenderFound(format!("No sender found for event {}", event)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use crate::message::ChaosMessageBuilder;
    use super::*;

    #[test]
    fn message_can_be_sent_to_listeners() {
        let mut communicator = ChaosCommunicator::new();
        let r = communicator.register_for(987654321);

        thread::spawn(move || {
            let message = ChaosMessageBuilder::new().with_param("test", 1123).build();
            assert_eq!(communicator.send_message(987654321, message).is_ok(), true);
        }).join().unwrap();

        assert_eq!(r.is_empty(), false);
        assert_eq!(r.recv().unwrap().get("test"), Some(1123));
    }

    #[test]
    fn sending_event_that_hasnt_been_registered_returns_error(){
        let communicator = ChaosCommunicator::new();
        let result = communicator.send_message("I dont exist", ChaosMessageBuilder::new().build());

        assert_eq!(result.is_err(), true);
    }
}