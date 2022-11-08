#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    NotImplementedError,
    InterpreterError,
    InvalidArgumentError,
}

// TODO(klinvill): consider using the thiserror crate instead.
#[derive(Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: None,
        }
    }

    pub fn with_message(kind: ErrorKind, message: String) -> Self {
        Error {
            kind,
            message: Some(message),
        }
    }
}
