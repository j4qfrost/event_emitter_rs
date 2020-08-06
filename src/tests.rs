use crate::EventEmitter;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, Arc};

#[test]
fn on() {
    let mut event_emitter = EventEmitter::new();
    let mut counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));

    let cloned_counter = Arc::clone(&counter);
    event_emitter.on("Set", move |value: u32| { 
        *cloned_counter.lock().unwrap() = value; 
    });

    let callbacks = event_emitter.emit("Set", 10 as u32);
    for callback in callbacks { // Wait for emitted callbacks to finish executing
        callback.join();
    }

    assert_eq!(
        10 as u32,
        *counter.lock().unwrap(),
        "Counter should have been set to the emitted value"
    );

    struct Container {
        list: Vec<String>
    }

    let mut container: Arc<Mutex<Container>> = Arc::new(Mutex::new(Container { list: Vec::new() }));

    let cloned_container = Arc::clone(&container);
    event_emitter.on("Add Value To List", move |value: String| { 
        let mut container = cloned_container.lock().unwrap(); 
        (*container).list.push(value);
    });

    let callbacks = event_emitter.emit("Add Value To List", "hello".to_string());
    for callback in callbacks { // Wait for emitted callbacks to finish executing
        callback.join();
    }

    assert_eq!(
        vec!["hello".to_string()],
        (*container.lock().unwrap()).list,
        "'hello' should have been pushed to the list after the 'Add Value To List' event was called with 'hello'"
    );
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

#[test]
fn on_limited() {
    let mut event_emitter = EventEmitter::new();
    let mut counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));

    let cloned_counter = Arc::clone(&counter);
    event_emitter.on_limited("Set", Some(2), move |value: u32| { 
        *cloned_counter.lock().unwrap() = value; 
    });
    assert_eq!(
        2,
        event_emitter.listeners.get("Set").unwrap().first().unwrap().limit.unwrap(),
        "Listener should have been added with a limit of 2 calls"
    );

    event_emitter.emit("Set", 10 as u32);
    assert_eq!(
        1,
        event_emitter.listeners.get("Set").unwrap().first().unwrap().limit.unwrap(),
        "Listener limit should have been reduced by 1"
    );

    event_emitter.emit("Set", 10 as u32);
    assert_eq!(
        0,
        event_emitter.listeners.get("Set").unwrap().first().unwrap().limit.unwrap(),
        "Listener should have 0 calls left"
    );

    event_emitter.emit("Set", 10 as u32);
    assert_eq!(
        0,
        event_emitter.listeners.get("Set").unwrap().len(),
        "Listener should have been removed"
    );
}

#[test]
fn once() {

}

mod event_emitter_file {
    use std::sync::Mutex;
    use crate::EventEmitter;
    
    lazy_static! {
        pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
    }
}

#[test]
fn global_emitter() {
    use event_emitter_file::EVENT_EMITTER;

    EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("hello there!"));
    EVENT_EMITTER.lock().unwrap().emit("Hello", ());
}