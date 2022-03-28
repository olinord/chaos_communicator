# chaos_communicator
A simple message sending arbitrary messages from one thread and 
receiving them else were for rust

Uses crossbeam to do the heavy lifting (message receive and sending)


### Usage

First we need to create a chaos_communicator
```rust
let mut communicator = ChaosCommunicator::new();
```

Then we can start registering for events that can be anything hashable and displayable
```rust    
#[derive(Hash)]
enum AppEvent{
    InterestingEvent = 1
}

impl fmt::Display for TestEnumEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

let receiver1 = communicator.register_for(123);
let receiver2 = communicator.register_for(AppEvent::InterestingEvent);
let receiver3 = communicator.register_for("event string");
```
And finally to send an event we can send arbitrary data over the wire. 
But to do that we need to construct a message
```rust
let message = ChaosMessageBuilder::new().
        with_param::<i32>("id", 1123).
        with_param::<u8>("age", 40).
        with_param::<&'static str>("name", "John Doe").
        build();
```
Which we can then send
```rust
communicator.send_message(AppEvent::InterestingEvent, message);
```
And receive
```rust
assert_eq!(receiver2.recv().unwrap().get("name"), Some("John Doe"))
```
    