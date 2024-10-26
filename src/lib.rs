use std::sync::{Arc, Mutex};

pub struct MacTest {
    duration: Arc<Mutex<f64>>,
}

impl MacTest {
    pub fn new() -> Self {
        Self {
            duration: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }
}
