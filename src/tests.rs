use crate::EventEmitter;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Foo {
    hello: u8,
    i_am: u8
}

#[test]
fn on() {
    let mut event_emitter = EventEmitter::new();
    event_emitter.on("Hello rust!", |value: f64| println!("\nHello world! {:#?}", value));

    assert_eq!(
        1,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Failed to add event emitter to listeners vector"
    );
    event_emitter.emit("Hello rust!", 12.0);

    event_emitter.on("Hello!", |value: f64| println!("\nHello worl1d! {:#?}", value));
    let foo = &105.0;
    event_emitter.emit("Hello!", foo);

    event_emitter.on("A", |number: f64| println!("\nnumber add {}", number + 3.0));
    let value = &5.0;
    event_emitter.emit("A", value); 
     // >> "8.0"

    // Using a more advanced type such as a struct
    #[derive(Serialize, Deserialize)]
    struct Date {
        month: String,
        day: String,   
    }

    let a = Date { month: "January".to_string(), day: "Tuesday".to_string() };
    event_emitter.on("LOG_DATE", |date: Date| println!("\n\n{} {}", date.day, date.month));
    event_emitter.emit("LOG_DATE", Date { month: "January".to_string(), day: "Tuesday".to_string() }); 
}

#[test]
fn remove_listener() {
    let mut event_emitter = EventEmitter::new();
    let listener_id = event_emitter.on("Hello rust!", |_: String| println!("Hello world!"));
    assert_eq!(
        1,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Failed to add event emitter to listeners vector"
    );

    event_emitter.remove_listener(&"foobar");
    assert_eq!(
        1,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Should not have removed listener"
    );

    event_emitter.remove_listener(&listener_id);
    assert_eq!(
        0,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Should have removed listener"
    );
}