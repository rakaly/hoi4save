/// A Hoi4 Error
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Hoi4Error(#[from] Box<Hoi4ErrorKind>);

impl Hoi4Error {
    pub(crate) fn new(kind: Hoi4ErrorKind) -> Hoi4Error {
        Hoi4Error(Box::new(kind))
    }

    /// Return the specific type of error
    pub fn kind(&self) -> &Hoi4ErrorKind {
        &self.0
    }
}

impl From<Hoi4ErrorKind> for Hoi4Error {
    fn from(err: Hoi4ErrorKind) -> Self {
        Hoi4Error::new(err)
    }
}

/// Specific type of error
#[derive(thiserror::Error, Debug)]
pub enum Hoi4ErrorKind {
    #[error("unable to parse due to: {0}")]
    Parse(#[source] jomini::Error),

    #[error("unable to deserialize due to: {0}")]
    Deserialize(#[source] jomini::Error),

    #[error("error while writing output: {0}")]
    Writer(#[source] jomini::Error),

    #[error("unknown binary token encountered: {token_id:#x}")]
    UnknownToken { token_id: u16 },

    #[error("expected the binary integer: {0} to be parsed as a date")]
    InvalidDate(i32),

    #[error("unknown header")]
    UnknownHeader,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_error_test() {
        assert_eq!(std::mem::size_of::<Hoi4Error>(), 8);
    }
}
