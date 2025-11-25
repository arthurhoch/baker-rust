#[derive(Clone)]
pub struct Logger {
    debug: bool,
}

impl Logger {
    pub fn new(debug: bool) -> Self {
        Self { debug }
    }

    pub fn log(&self, message: &str) {
        println!("{}", message);
    }

    pub fn debug(&self, message: &str) {
        if self.debug {
            eprintln!("DEBUG: {}", message);
        }
    }
}
