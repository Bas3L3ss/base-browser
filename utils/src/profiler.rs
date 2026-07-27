use std::{format, println, time::Instant};

pub struct Profile {
    name: &'static str,
    start: Instant,
}

impl Profile {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        println!(
            "[PROFILE] {:<12} {}",
            self.name,
            format!("{:?}", self.start.elapsed())
        );
    }
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _profile = $crate::profiler::Profile::new($name);
    };
}
