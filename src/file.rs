use crate::{flavor::Hoi4Flavor, Encoding, Hoi4Error, Hoi4ErrorKind, Hoi4Melter};
use jomini::{
    binary::{BinaryDeserializerBuilder, FailedResolveStrategy, TokenResolver},
    text::ObjectReader,
    BinaryDeserializer, BinaryTape, TextDeserializer, TextTape, Utf8Encoding,
};
use serde::Deserialize;

const TXT_HEADER: &[u8] = b"HOI4txt";
const BIN_HEADER: &[u8] = b"HOI4bin";

fn is_text(data: &[u8]) -> Option<&[u8]> {
    let sentry = TXT_HEADER;
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

fn is_bin(data: &[u8]) -> Option<&[u8]> {
    let sentry = BIN_HEADER;
    if data.get(..sentry.len()).map_or(false, |x| x == sentry) {
        Some(&data[sentry.len()..])
    } else {
        None
    }
}

enum FileKind<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
}

/// Entrypoint for parsing HOI4 saves
///
/// Only consumes enough data to determine encoding of the file
pub struct Hoi4File<'a> {
    kind: FileKind<'a>,
}

impl<'a> Hoi4File<'a> {
    /// Creates a HOI4 file from a slice of data
    pub fn from_slice(data: &[u8]) -> Result<Hoi4File, Hoi4Error> {
        if let Some(text_data) = is_text(data) {
            Ok(Hoi4File {
                kind: FileKind::Text(text_data),
            })
        } else if let Some(bin_data) = is_bin(data) {
            Ok(Hoi4File {
                kind: FileKind::Binary(bin_data),
            })
        } else {
            Err(Hoi4Error::new(Hoi4ErrorKind::UnknownHeader))
        }
    }

    /// Returns the detected decoding of the file
    pub fn encoding(&self) -> Encoding {
        match &self.kind {
            FileKind::Text(_) => Encoding::Plaintext,
            FileKind::Binary(_) => Encoding::Binary,
        }
    }

    /// Returns the size of the file
    ///
    /// The size includes the inflated size of the zip
    pub fn size(&self) -> usize {
        match &self.kind {
            FileKind::Text(x) | FileKind::Binary(x) => x.len(),
        }
    }

    /// Parse save
    pub fn parse(&self) -> Result<Hoi4ParsedFile<'a>, Hoi4Error> {
        match &self.kind {
            FileKind::Text(x) => {
                let text = Hoi4Text::from_raw(x)?;
                Ok(Hoi4ParsedFile {
                    kind: Hoi4ParsedFileKind::Text(text),
                })
            }
            FileKind::Binary(x) => {
                let binary = Hoi4Binary::from_raw(x)?;
                Ok(Hoi4ParsedFile {
                    kind: Hoi4ParsedFileKind::Binary(binary),
                })
            }
        }
    }
}

/// Contains the parsed Hoi4 file
pub enum Hoi4ParsedFileKind<'a> {
    /// The Hoi4 file as text
    Text(Hoi4Text<'a>),

    /// The Hoi4 file as binary
    Binary(Hoi4Binary<'a>),
}

/// An Hoi4 file that has been parsed
pub struct Hoi4ParsedFile<'a> {
    kind: Hoi4ParsedFileKind<'a>,
}

impl<'a> Hoi4ParsedFile<'a> {
    /// Returns the file as text
    pub fn as_text(&self) -> Option<&Hoi4Text> {
        match &self.kind {
            Hoi4ParsedFileKind::Text(x) => Some(x),
            _ => None,
        }
    }

    /// Returns the file as binary
    pub fn as_binary(&self) -> Option<&Hoi4Binary> {
        match &self.kind {
            Hoi4ParsedFileKind::Binary(x) => Some(x),
            _ => None,
        }
    }

    /// Returns the kind of file (binary or text)
    pub fn kind(&self) -> &Hoi4ParsedFileKind {
        &self.kind
    }

    /// Prepares the file for deserialization into a custom structure
    pub fn deserializer(&self) -> Hoi4Deserializer {
        match &self.kind {
            Hoi4ParsedFileKind::Text(x) => Hoi4Deserializer {
                kind: Hoi4DeserializerKind::Text(x),
            },
            Hoi4ParsedFileKind::Binary(x) => Hoi4Deserializer {
                kind: Hoi4DeserializerKind::Binary(x.deserializer()),
            },
        }
    }
}

/// A parsed Hoi4 text document
pub struct Hoi4Text<'a> {
    tape: TextTape<'a>,
}

impl<'a> Hoi4Text<'a> {
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Hoi4Error> {
        is_text(data)
            .ok_or_else(|| Hoi4ErrorKind::UnknownHeader.into())
            .and_then(Self::from_raw)
    }

    pub(crate) fn from_raw(data: &'a [u8]) -> Result<Self, Hoi4Error> {
        let tape = TextTape::from_slice(data).map_err(Hoi4ErrorKind::Parse)?;
        Ok(Hoi4Text { tape })
    }

    pub fn reader(&self) -> ObjectReader<Utf8Encoding> {
        self.tape.utf8_reader()
    }

    pub fn deserialize<T>(&self) -> Result<T, Hoi4Error>
    where
        T: Deserialize<'a>,
    {
        let result =
            TextDeserializer::from_utf8_tape(&self.tape).map_err(Hoi4ErrorKind::Deserialize)?;
        Ok(result)
    }
}

/// A parsed Hoi4 binary document
pub struct Hoi4Binary<'a> {
    tape: BinaryTape<'a>,
}

impl<'a> Hoi4Binary<'a> {
    pub fn from_slice(data: &'a [u8]) -> Result<Self, Hoi4Error> {
        is_bin(data)
            .ok_or_else(|| Hoi4ErrorKind::UnknownHeader.into())
            .and_then(Self::from_raw)
    }

    pub(crate) fn from_raw(data: &'a [u8]) -> Result<Self, Hoi4Error> {
        let tape = BinaryTape::from_slice(data).map_err(Hoi4ErrorKind::Parse)?;
        Ok(Hoi4Binary { tape })
    }

    pub fn deserializer<'b>(&'b self) -> Hoi4BinaryDeserializer<'a, 'b> {
        Hoi4BinaryDeserializer {
            builder: BinaryDeserializer::builder_flavor(Hoi4Flavor),
            tape: &self.tape,
        }
    }

    pub fn melter<'b>(&'b self) -> Hoi4Melter<'a, 'b> {
        Hoi4Melter::new(&self.tape)
    }
}

enum Hoi4DeserializerKind<'a, 'b> {
    Text(&'b Hoi4Text<'a>),
    Binary(Hoi4BinaryDeserializer<'a, 'b>),
}

/// A deserializer for custom structures
pub struct Hoi4Deserializer<'a, 'b> {
    kind: Hoi4DeserializerKind<'a, 'b>,
}

impl<'a, 'b> Hoi4Deserializer<'a, 'b> {
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        if let Hoi4DeserializerKind::Binary(x) = &mut self.kind {
            x.on_failed_resolve(strategy);
        }
        self
    }

    pub fn build<T, R>(&self, resolver: &'a R) -> Result<T, Hoi4Error>
    where
        R: TokenResolver,
        T: Deserialize<'a>,
    {
        match &self.kind {
            Hoi4DeserializerKind::Text(x) => x.deserialize(),
            Hoi4DeserializerKind::Binary(x) => x.build(resolver),
        }
    }
}

/// Deserializes binary data into custom structures
pub struct Hoi4BinaryDeserializer<'a, 'b> {
    builder: BinaryDeserializerBuilder<Hoi4Flavor>,
    tape: &'b BinaryTape<'a>,
}

impl<'a, 'b> Hoi4BinaryDeserializer<'a, 'b> {
    pub fn on_failed_resolve(&mut self, strategy: FailedResolveStrategy) -> &mut Self {
        self.builder.on_failed_resolve(strategy);
        self
    }

    pub fn build<T, R>(&self, resolver: &'a R) -> Result<T, Hoi4Error>
    where
        R: TokenResolver,
        T: Deserialize<'a>,
    {
        let result = self
            .builder
            .from_tape(self.tape, resolver)
            .map_err(|e| match e.kind() {
                jomini::ErrorKind::Deserialize(e2) => match e2.kind() {
                    &jomini::DeserializeErrorKind::UnknownToken { token_id } => {
                        Hoi4ErrorKind::UnknownToken { token_id }
                    }
                    _ => Hoi4ErrorKind::Deserialize(e),
                },
                _ => Hoi4ErrorKind::Deserialize(e),
            })?;
        Ok(result)
    }
}
