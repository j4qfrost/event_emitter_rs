//! A simple EventEmitter implementation.
//!
//! Allows you to subscribe to events with callbacks and also fire those events.
//! Events are in the form of (strings, value) and callbacks are in the form of closures that take in a value parameter.

//! # Basic Usage
//!
//! ```
//! use event_emitter::EventEmitter;
//! let mut event_emitter = EventEmitter::new();
//!
//! // This will print <"Hello world!"> whenever the <"Say Hello"> event is emitted
//! event_emitter.on("Say Hello", |value: ()| println!("Hello world!"));
//! event_emitter.emit("Say Hello", ()); 
//! // >> "Hello world!"
//! ```
//!
//! # Advanced Usage
//!
//! We can also emit values of any type so long as they implement the serde Serialize and Deserialize traits
//!
//! ```
//! use event_emitter::EventEmitter;
//! let mut event_emitter = EventEmitter::new();
//!
//! event_emitter.on("Add three", |number: f32| println!("{}", number + 3.0));
//! event_emitter.emit("Add three", 5.0 as f32); 
//! // >> "8.0"
//!
//!
//! // Using a more advanced value type such as a struct by implementing the serde traits
//! #[derive(Serialize, Deserialize)]
//! struct Date {
//!     month: String,
//!     day: String,   
//! }
//!
//! event_emitter.on("LOG_DATE", |date: Date| {
//!     println!("Month: {} - Day: {}", date.month, date.day)
//! });
//! event_emitter.emit("LOG_DATE", Date { 
//!     month: "January".to_string(), 
//!     day: "Tuesday".to_string() 
//! }); 
//! // >> "Month: January - Day: Tuesday"
//! ```
//!
//! Removing listeners is also easy
//!
//! ```
//! use event_emitter::EventEmitter;
//! let mut event_emitter = EventEmitter::new();
//!
//! let listener_id = event_emitter.on("Hello", |_: ()| println!("Hello World"));
//! match event_emitter.remove_listener(listener_id) {
//!     Some(listener_id) => print!("Removed event listener!"),
//!     None => print!("No event listener of that id exists")
//! }
//! ```

//#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::thread;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use bincode;

use uuid::Uuid;


#[derive(Default)]
pub struct EventEmitter {
    pub listeners: HashMap<String, Vec<(String, Arc<dyn Fn(Vec<u8>) + 'static + Sync + Send>)>>
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
    /// use event_emitter::EventEmitter;
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
        let id = Uuid::new_v4().to_string();
        let parsed_callback = move |bytes: Vec<u8>| {
            let value: T = bincode::deserialize(&bytes).unwrap();
            callback(value);
        };

        match self.listeners.get_mut(event) {
            Some(callbacks) => { callbacks.push((id.clone(), Arc::new(parsed_callback))); },
            None => { self.listeners.insert(event.to_string(), vec![(id.clone(), Arc::new(parsed_callback))]); }
        }

        return id;
    }

    /// Emits an event of the given parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// // Emits the <"Some event"> event and a value <"Hello programmer">
    /// // The value can be of any type
    /// event_emitter.emit("Some event", "Hello programmer!");
    /// ```
    pub fn emit<T>(&self, event: &str, value: T) 
        where T: Serialize
    {
        if let Some(callbacks) = self.listeners.get(event) {
            let bytes: Vec<u8> = bincode::serialize(&value).unwrap();
            for callback in callbacks.iter().map(|(_, callback)| Arc::clone(callback)) {
                let cloned_bytes = bytes.clone();
                thread::spawn(move || callback(cloned_bytes));
            }
        }
    }

    /// Removes an event listener with the given id
    ///
    /// # Example
    ///
    /// ```
    /// use event_emitter::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    /// let listener_id = event_emitter.on("Some event", |value: ()| println!("Hello world!"));
    ///
    /// // Removes the listener that we just added
    /// event_emitter.remove_listener(&listener_id);
    /// ```
    pub fn remove_listener(&mut self, id_to_delete: &str) -> Option<String> {
        for (_, event_listeners) in self.listeners.iter_mut() {
            if let Some(index) = event_listeners.iter().position(|(id, _)| id == id_to_delete) {
                event_listeners.remove(index);
                return Some(id_to_delete.to_string());
            } 
        }

        return None;
    }
}