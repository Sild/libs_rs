use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub wait_duration: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wait_duration: Duration::MAX,
        }
    }
}
