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

pub fn new_warn(message: &str) -> Box<dyn Error> {
    log::warn!("{}", message);
    Box::new(AnyError(message.to_owned()))
}

pub fn new_err(message: &str) -> Box<dyn Error> {
    log::error!("{}", message);
    Box::new(AnyError(message.to_owned()))
}