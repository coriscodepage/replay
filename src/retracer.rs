use std::{collections::BTreeMap, error::Error, fmt::Display, panic::Location};

use crate::call::Call;

pub type Callback = fn(&mut Call);

struct Entry {
    name: String,
    callback: Callback,
}

#[derive(Debug)]
pub enum RetracerError {
    NoCallback(&'static Location<'static>),
}

impl RetracerError {
    #[track_caller]
    pub fn no_callback() -> Self {
        Self::NoCallback(Location::caller())
    }
}

impl Display for RetracerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetracerError::NoCallback(location) => write!(f, "RetracerError error: NoCallback at {}:{}", location.file(), location.line()),
        }
    }
}

impl Error for RetracerError {}

pub struct Retracer {
    map: BTreeMap<String, Callback>,
    callbacks: Vec<Option<Callback>>,
}

impl Retracer {
    pub fn init() -> Self {
        Self { map: BTreeMap::new(), callbacks: Vec::new() }
    }

    pub fn retrace(&mut self, call: &mut Call) -> Result<(), RetracerError>{
        let mut callback: Option<Callback> = None;
        let id = call.sig.id;
        if id >= self.callbacks.len() {
            self.callbacks.resize(id + 1, None);
        }
        else {
            callback = self.callbacks[id];
        }

        if callback.is_none() {
            callback = self.map.get(&call.sig.name).copied();
            self.callbacks[id] = callback;
        }
        if let Some(callback) = callback {
            callback(call);
            Ok(())
        }
        else {
            Err(RetracerError::no_callback())
        }
    }

    fn add_calback(&mut self, entry: Entry) {
        self.map.insert(entry.name, entry.callback);
    }

}
