#[derive(Debug)]
pub enum ErrorKind {
    NotImplementedError,
}

// TODO(klinvill): consider using the thiserror crate instead.
#[derive(Debug)]
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
}
