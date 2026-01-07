use std::io;

pub enum IntoEventStreamError {
    IO(io::Error),
    NiriNotHandled(String),
}

impl From<io::Error> for IntoEventStreamError {
    fn from(value: io::Error) -> Self {
        IntoEventStreamError::IO(value)
    }
}
