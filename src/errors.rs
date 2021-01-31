use std::fmt;
use std::io::Error as IoError;

/// An Hoi4 Error
#[derive(Debug)]
pub struct Hoi4Error(Box<Hoi4ErrorKind>);

impl Hoi4Error {
    pub(crate) fn new(kind: Hoi4ErrorKind) -> Hoi4Error {
        Hoi4Error(Box::new(kind))
    }

    /// Return the specific type of error
    pub fn kind(&self) -> &Hoi4ErrorKind {
        &self.0
    }
}

/// Specific type of error
#[derive(Debug)]
pub enum Hoi4ErrorKind {
    IoErr(IoError),
    UnknownHeader,
    UnknownToken {
        token_id: u16,
    },
    Deserialize {
        part: Option<String>,
        err: jomini::Error,
    },
}

impl fmt::Display for Hoi4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            Hoi4ErrorKind::IoErr(_) => write!(f, "io error"),
            Hoi4ErrorKind::UnknownHeader => write!(f, "unknown header encountered in zip"),
            Hoi4ErrorKind::UnknownToken { token_id } => {
                write!(f, "unknown binary token encountered (id: {})", token_id)
            }
            Hoi4ErrorKind::Deserialize { ref part, ref err } => match part {
                Some(p) => write!(f, "error deserializing: {}: {}", p, err),
                None => err.fmt(f),
            },
        }
    }
}

impl std::error::Error for Hoi4Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind() {
            Hoi4ErrorKind::IoErr(e) => Some(e),
            Hoi4ErrorKind::Deserialize { ref err, .. } => Some(err),
            _ => None,
        }
    }
}

impl From<jomini::Error> for Hoi4Error {
    fn from(err: jomini::Error) -> Self {
        Hoi4Error::new(Hoi4ErrorKind::Deserialize { part: None, err })
    }
}

impl From<IoError> for Hoi4Error {
    fn from(err: IoError) -> Self {
        Hoi4Error::new(Hoi4ErrorKind::IoErr(err))
    }
}

impl From<Hoi4ErrorKind> for Hoi4Error {
    fn from(err: Hoi4ErrorKind) -> Self {
        Hoi4Error::new(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_error_test() {
        assert_eq!(std::mem::size_of::<Hoi4Error>(), 8);
    }
}
