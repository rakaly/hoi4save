use jomini::{BinaryDeserializer, FailedResolveStrategy, TextDeserializer, TextTape};

use crate::{flavor::Hoi4Flavor, models::Hoi4Save, tokens::TokenLookup, Hoi4Error};

/// Describes the format of the save before decoding
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Encoding {
    /// Binary save
    Binary,

    /// Plaintext save
    Plaintext,
}

/// Customize how a save is extracted
#[derive(Debug, Clone)]
pub struct Hoi4ExtractorBuilder {
    on_failed_resolve: FailedResolveStrategy,
}

impl Default for Hoi4ExtractorBuilder {
    fn default() -> Self {
        Hoi4ExtractorBuilder::new()
    }
}

impl Hoi4ExtractorBuilder {
    /// Create a new extractor with default values: extract zips into memory
    // and ignore unknown binary tokens
    pub fn new() -> Self {
        Hoi4ExtractorBuilder {
            on_failed_resolve: FailedResolveStrategy::Ignore,
        }
    }

    /// Set the behavior for when an unresolved binary token is encountered
    pub fn with_on_failed_resolve(mut self, strategy: FailedResolveStrategy) -> Self {
        self.on_failed_resolve = strategy;
        self
    }

    /// Extract all info from a save
    pub fn extract_save(&self, data: &[u8]) -> Result<(Hoi4Save, Encoding), Hoi4Error> {
        self.extract_save_as(data)
    }

    // todo, customize deserialize type
    fn extract_save_as(&self, data: &[u8]) -> Result<(Hoi4Save, Encoding), Hoi4Error> {
        let text_header = b"HOI4txt";
        let binary_header = b"HOI4bin";
        match data.get(..text_header.len()) {
            Some(x) if x == text_header => {
                let save_data = &data[text_header.len()..];
                let tape = TextTape::from_slice(save_data)?;
                let save = TextDeserializer::from_utf8_tape(&tape)?;
                Ok((save, Encoding::Plaintext))
            }
            Some(x) if x == binary_header => {
                let save_data = &data[binary_header.len()..];
                let save = BinaryDeserializer::builder_flavor(Hoi4Flavor)
                    .from_slice(save_data, &TokenLookup)?;
                Ok((save, Encoding::Binary))
            }
            _ => return Err(Hoi4Error::new(crate::Hoi4ErrorKind::UnknownHeader)),
        }
    }
}

/// Logic container for extracting data from an Hoi4 save
#[derive(Debug, Clone)]
pub struct Hoi4Extractor {}

impl Hoi4Extractor {
    /// Create a customized container
    pub fn builder() -> Hoi4ExtractorBuilder {
        Hoi4ExtractorBuilder::new()
    }

    /// Extract all info from a save
    pub fn extract_save(&self, data: &[u8]) -> Result<(Hoi4Save, Encoding), Hoi4Error> {
        Self::builder().extract_save(data)
    }
}
