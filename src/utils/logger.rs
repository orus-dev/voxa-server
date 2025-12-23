use std::fmt::Display;

pub struct Logger {
    name: String,
}

impl Logger {
    pub fn new<T: Display>(name: T) -> Self {
        Logger {
            name: name.to_string(),
        }
    }

    pub fn info<T: Display>(&self, message: T) {
        println!("\x1b[32mINFO\x1b[0m ({}) › {}", self.name, message);
    }

    pub fn warn<T: Display>(&self, message: T) {
        println!("\x1b[33mWARN\x1b[0m ({}) › {}", self.name, message);
    }

    pub fn error<T: Display>(&self, message: T) {
        println!("\x1b[31mERROR\x1b[0m ({}) › {}", self.name, message);
    }

    pub fn extract<T, E: Display, D: Display>(&self, v: Result<T, E>, m: D) -> Option<T> {
        match v {
            Ok(a) => Some(a),
            Err(e) => {
                self.error(format!("{m}: {e}"));
                None
            }
        }
    }

    // pub fn extract_panic<T, E: Display, D: Display>(&self, v: Result<T, E>, m: D) -> T {
    //     match v {
    //         Ok(a) => a,
    //         Err(e) => {
    //             self.error(format!("{m}: {e}"));
    //             panic!()
    //         }
    //     }
    // }
}
