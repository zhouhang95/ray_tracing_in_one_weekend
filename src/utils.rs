use std::time::Instant;

pub struct EZTimer {
    start: Instant,
}

impl EZTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for EZTimer{
    fn drop(&mut self) {
        eprintln!("elapsed {} ms", self.start.elapsed().as_millis());
    }
}
