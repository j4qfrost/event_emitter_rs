/*! 
    A simple EventEmitter implementation.

    Allows you to subscribe to events with callbacks and also fire those events.
    Events are in the form of (strings, value) and callbacks are in the form of closures that take in a value parameter.

    # Getting Started

    ```
    use event_emitter_rs::EventEmitter;
    let mut event_emitter = EventEmitter::new();

    // This will print <"Hello world!"> whenever the <"Say Hello"> event is emitted
    event_emitter.on("Say Hello", |value: ()| println!("Hello world!"));
    event_emitter.emit("Say Hello", ()); 
    // >> "Hello world!"
    ```

    # Basic Usage

    We can emit values of any type so long as they implement the serde Serialize and Deserialize traits.
    A single EventEmitter instance can have listeners to values of multiple types.

    ```
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

    ```
    use event_emitter_rs::EventEmitter;
    let mut event_emitter = EventEmitter::new();

    let listener_id = event_emitter.on("Hello", |_: ()| println!("Hello World"));
    match event_emitter.remove_listener(&listener_id) {
        Some(listener_id) => print!("Removed event listener!"),
        None => print!("No event listener of that id exists")
    }
    ```
    # Creating a Global EventEmitter

    It's likely that you'll want to have a single EventEmitter instance that can be shared accross files;

    After all, one of the main points of using an EventEmitter is to avoid passing down a value through several nested functions/types and having a global subscription service.

    ```
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

    ```
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

    ```
    // some_random_file.rs
    use crate::global_event_emitter::EVENT_EMITTER;

    fn random_function() {
        // When the <"Hello"> event is emitted in main.rs then print <"Random stuff!">
        EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("Random stuff!"));
    }
    ```
*/

//#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::thread;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate lazy_static;

use bincode;

use uuid::Uuid;

pub struct Listener {
    callback: Arc<dyn Fn(Vec<u8>) + Sync + Send + 'static>,
    limit: Option<u64>,
    id: String,
}

#[derive(Default)]
pub struct EventEmitter {
    pub listeners: HashMap<String, Vec<Listener>>
}

impl EventEmitter {
    // Potentially may want to add features here in the future so keep it like this
    pub fn new() -> Self {
        Self {
            ..Self::default()
        }
    }

    /// Adds an event listener with a callback that will get called whenever the given event is emitted.
    /// Returns the id of the newly added listener.
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// // This will print <"Hello world!"> whenever the <"Some event"> event is emitted
    /// // type of the value parameter for the closure MUST be specified (here we just use a throwaway `()` type)
    /// event_emitter.on("Some event", |value: ()| println!("Hello world!"));
    /// ```
    pub fn on<F, T>(&mut self, event: &str, callback: F) -> String
        where 
            for<'de> T: Deserialize<'de>,
            F: Fn(T) + 'static + Sync + Send 
    {
        let id = self.on_limited(event, callback, None);
        return id;
    }

    pub fn on_limited<F, T>(&mut self, event: &str, callback: F, limit: Option<u64>) -> String
        where 
            for<'de> T: Deserialize<'de>,
            F: Fn(T) + 'static + Sync + Send 
    {
        let id = Uuid::new_v4().to_string();
        let parsed_callback = move |bytes: Vec<u8>| {
            let value: T = bincode::deserialize(&bytes).unwrap();
            callback(value);
        };

        let listener = Listener {
            id: id.clone(),
            limit,
            callback: Arc::new(parsed_callback),
        };

        match self.listeners.get_mut(event) {
            Some(callbacks) => { callbacks.push(listener); },
            None => { self.listeners.insert(event.to_string(), vec![listener]); }
        }

        return id;
    }

    pub fn once<F, T>(&mut self, event: &str, callback: F) -> String
        where 
            for<'de> T: Deserialize<'de>,
            F: Fn(T) + 'static + Sync + Send 
    {
        let id = self.on_limited(event, callback, Some(1));
        return id;
    }

    /// Emits an event of the given parameters and executes each callback that is listening to that event asynchronously by spawning a new thread for each callback.
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// // Emits the <"Some event"> event and a value <"Hello programmer">
    /// // The value can be of any type
    /// event_emitter.emit("Some event", "Hello programmer!");
    /// ```
    pub fn emit<T>(&mut self, event: &str, value: T) 
        where T: Serialize
    {
        if let Some(listeners) = self.listeners.get_mut(event) {
            let bytes: Vec<u8> = bincode::serialize(&value).unwrap();
            
            let mut listeners_to_remove: Vec<usize> = Vec::new();
            for (index, listener) in listeners.iter_mut().enumerate() {
                let cloned_bytes = bytes.clone();
                let callback = Arc::clone(&listener.callback);

                match listener.limit {
                    None => { thread::spawn(move || callback(cloned_bytes)); },
                    Some(limit) => {
                        if limit != 0 {
                            thread::spawn(move || callback(cloned_bytes));
                            listener.limit = Some(limit - 1);
                        } else {
                            listeners_to_remove.push(index);
                        }
                    }
                }
            }

            // Reverse here so we don't mess up the ordering of the vector
            for index in listeners_to_remove.into_iter().rev() {
                listeners.remove(index);
            }
        }
    }

    /// Emits an event of the given parameters in a synchronous fashion.
    /// Instead of executing each callback in a newly spawned thread, it will execute each callback in the order that they were inserted.
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.on("Some event", |value: ()| println!("1")); // Guaranteed to be executed first
    /// event_emitter.on("Some event", |value: ()| println!("2")); // Will not execute this until the first callback has finished executing
    /// event_emitter.on("Some event", |value: ()| println!("3")); // Will not execute this until the second callback has finished executing
    ///
    /// // Emits the <"Some event"> event and a value <"Hello programmer">
    /// // The value can be of any type
    /// event_emitter.sync_emit("Some event", "Hello programmer!");
    /// ```
    pub fn sync_emit<T>(&self, event: &str, value: T) 
        where T: Serialize
    {
    }

    /// Removes an event listener with the given id
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    /// let listener_id = event_emitter.on("Some event", |value: ()| println!("Hello world!"));
    ///
    /// // Removes the listener that we just added
    /// event_emitter.remove_listener(&listener_id);
    /// ```
    pub fn remove_listener(&mut self, id_to_delete: &str) -> Option<String> {
        for (_, event_listeners) in self.listeners.iter_mut() {
            if let Some(index) = event_listeners.iter().position(|listener| listener.id == id_to_delete) {
                event_listeners.remove(index);
                return Some(id_to_delete.to_string());
            } 
        }

        return None;
    }
}