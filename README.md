# event-emitter-rs

A simple EventEmitter implementation.

Allows you to subscribe to events with callbacks and also fire those events.
Events are in the form of (strings, value) and callbacks are in the form of closures that take in a value parameter.

## Getting Started

```rust
use event_emitter_rs::EventEmitter;
let mut event_emitter = EventEmitter::new();

// This will print <"Hello world!"> whenever the <"Say Hello"> event is emitted
event_emitter.on("Say Hello", |value: ()| println!("Hello world!"));
event_emitter.emit("Say Hello", ());
// >> "Hello world!"
```

## Basic Usage

We can emit and listen to values of any type so long as they implement the serde Serialize and Deserialize traits.
A single EventEmitter instance can have listeners to values of multiple types.

```rust
use event_emitter_rs::EventEmitter;
use serde::{Deserialize, Serialize};
let mut event_emitter = EventEmitter::new();

event_emitter.on("Add three", |number: f32| println!("{}", number + 3.0));
event_emitter.emit("Add three", 5.0 as f32);
event_emitter.emit("Add three", 4.0 as f32);
// >> "8.0"
// >> "7.0"

// Using a more advanced value type such as a struct by implementing the serde traits
#[derive(Serialize, Deserialize)]
struct Date {
    month: String,
    day: String,
}

event_emitter.on("LOG_DATE", |date: Date| {
    println!("Month: {} - Day: {}", date.month, date.day)
});
event_emitter.emit("LOG_DATE", Date {
    month: "January".to_string(),
    day: "Tuesday".to_string()
});
// >> "Month: January - Day: Tuesday"
```

Removing listeners is also easy

```rust
use event_emitter_rs::EventEmitter;
let mut event_emitter = EventEmitter::new();

let listener_id = event_emitter.on("Hello", |_: ()| println!("Hello World"));
match event_emitter.remove_listener(&listener_id) {
    Some(listener_id) => print!("Removed event listener!"),
    None => print!("No event listener of that id exists")
}
```
## Creating a Global EventEmitter

It's likely that you'll want to have a single EventEmitter instance that can be shared accross files;

After all, one of the main points of using an EventEmitter is to avoid passing down a value through several nested functions/types and having a global subscription service.

```rust
// global_event_emitter.rs
use std::sync::Mutex;
use crate::EventEmitter;

// Use lazy_static! because the size of EventEmitter is not known at compile time
lazy_static! {
    // Export the emitter with `pub` keyword
    pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
}
```

Then we can import this instance into multiple files.

```rust
// main.rs
#[macro_use]
extern crate lazy_static;

mod global_event_emitter;
use global_event_emitter::EVENT_EMITTER;

fn main() {
    // We need to maintain a lock through the mutex so we can avoid data races
    EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("hello there!"));
    EVENT_EMITTER.lock().unwrap().emit("Hello", ());
}
```

And in another file we can now listen to the <"Hello"> event in main.rs by adding a listener to the global event emitter.

```rust
// some_random_file.rs
use crate::global_event_emitter::EVENT_EMITTER;

fn random_function() {
    // When the <"Hello"> event is emitted in main.rs then print <"Random stuff!">
    EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("Random stuff!"));
}
```

License: MIT OR Apache-2.0
