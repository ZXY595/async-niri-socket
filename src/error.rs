use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NiriReplyError {
    #[error("Failed to communicate with Niri due to IO error: {0}")]
    IO(io::Error),
    /// Some error message from Niri.
    #[error("Niri reply with error message: {0}")]
    Niri(String),
}

impl NiriReplyError {
    /// Return the IO error if there is any.
    pub fn io_error(self) -> Option<io::Error> {
        if let NiriReplyError::IO(err) = self {
            Some(err)
        } else {
            None
        }
    }

    /// Return the Niri error message if there is any.
    pub fn niri_error_msg(self) -> Option<String> {
        if let NiriReplyError::Niri(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}

impl From<io::Error> for NiriReplyError {
    fn from(value: io::Error) -> Self {
        NiriReplyError::IO(value)
    }
}
