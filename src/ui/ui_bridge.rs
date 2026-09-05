use std::collections::HashMap;

pub struct UiBridge {
    callbacks: HashMap<String, Box<dyn Fn(&str) -> ()>>,
}

impl UiBridge {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
        }
    }

    pub fn register_callback<F>(&mut self, name: &str, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        self.callbacks.insert(name.to_string(), Box::new(callback));
    }

    pub fn call(&self, name: &str, data: &str) {
        if let Some(callback) = self.callbacks.get(name) {
            callback(data);
        } else {
            log::warn!("Unknown UI callback: {}", name);
        }
    }
}
