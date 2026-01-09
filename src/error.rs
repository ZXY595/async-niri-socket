use std::io;

#[derive(Debug)]
pub enum NiriReplyError {
    /// Fail to communicate with Niri because of IO error.
    IO(io::Error),
    /// Some error message from Niri.
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
