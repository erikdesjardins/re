use std::io;

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
