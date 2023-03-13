use std::fmt::{self, Debug, Display};
use std::io;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct DisplayError(Error);

impl Debug for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Into<Error>> From<T> for DisplayError {
    fn from(display: T) -> Self {
        DisplayError(display.into())
    }
}

pub trait IoErrorExt {
    fn applies_to(&self) -> AppliesTo;
}

impl IoErrorExt for io::Error {
    fn applies_to(&self) -> AppliesTo {
        match self.kind() {
            io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset => AppliesTo::Connection,
            _ => AppliesTo::Listener,
        }
    }
}

pub enum AppliesTo {
    Connection,
    Listener,
}
