use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct AnyError(String);

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AnyError {}

pub fn new_err(message: &str) -> Box<dyn Error> {
    Box::new(AnyError(message.to_owned()))
}