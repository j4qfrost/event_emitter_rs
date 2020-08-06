# event_emitter

A simple EventEmitter implementation.

Allows you to subscribe to events with callbacks and also fire those events.
Events are in the form of (strings, value) and callbacks are in the form of closures that take in a value parameter.
## Basic Usage

```rust
use event_emitter::EventEmitter;
let mut event_emitter = EventEmitter::new();

// This will print <"Hello world!"> whenever the <"Say Hello"> event is emitted
event_emitter.on("Say Hello", |value: ()| println!("Hello world!"));
event_emitter.emit("Say Hello", ());
// >> "Hello world!"
```

## Advanced Usage

We can also emit values of any type so long as they implement the serde Serialize and Deserialize traits

```rust
use event_emitter::EventEmitter;
let mut event_emitter = EventEmitter::new();

event_emitter.on("Add three", |number: f32| println!("{}", number + 3.0));
event_emitter.emit("Add three", 5.0 as f32);
// >> "8.0"


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
use event_emitter::EventEmitter;
let mut event_emitter = EventEmitter::new();

let listener_id = event_emitter.on("Hello", |_: ()| println!("Hello World"));
match event_emitter.remove_listener(listener_id) {
    Some(listener_id) => print!("Removed event listener!"),
    None => print!("No event listener of that id exists")
}
```
