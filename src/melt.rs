use crate::{
    flavor::Hoi4Flavor, tokens::TokenLookup, FailedResolveStrategy, Hoi4Date, Hoi4Error,
    Hoi4ErrorKind, PdsDate,
};
use jomini::{BinaryFlavor, BinaryTape, BinaryToken, TextWriterBuilder, TokenResolver};
use std::collections::HashSet;

/// Convert a binary gamestate to plaintext
#[derive(Debug)]
pub struct Melter {
    on_failed_resolve: FailedResolveStrategy,
    rewrite: bool,
}

impl Default for Melter {
    fn default() -> Self {
        Melter {
            on_failed_resolve: FailedResolveStrategy::Ignore,
            rewrite: true,
        }
    }
}

impl Melter {
    /// Create a customized version to melt binary data
    pub fn new() -> Self {
        Melter::default()
    }

    /// Set the behavior for when an unresolved binary token is encountered
    pub fn with_on_failed_resolve(mut self, strategy: FailedResolveStrategy) -> Self {
        self.on_failed_resolve = strategy;
        self
    }

    /// Set if the melter should rewrite properties to better match the plaintext format
    ///
    /// Setting to false will preserve binary fields and values even if they
    /// don't make any sense in the plaintext output.
    pub fn with_rewrite(mut self, rewrite: bool) -> Self {
        self.rewrite = rewrite;
        self
    }

    fn convert<Q>(
        &self,
        input: &[u8],
        writer: &mut Vec<u8>,
        unknown_tokens: &mut HashSet<u16>,
        resolver: &Q,
    ) -> Result<(), Hoi4Error>
    where
        Q: TokenResolver,
    {
        let flavor = Hoi4Flavor;
        let tape = BinaryTape::from_slice(input)?;
        let mut wtr = TextWriterBuilder::new()
            .indent_char(b'\t')
            .indent_factor(1)
            .from_writer(writer);
        let mut token_idx = 0;
        let mut known_number = false;
        let mut known_date = false;
        let tokens = tape.tokens();

        while let Some(token) = tokens.get(token_idx) {
            match token {
                BinaryToken::Object(_) => {
                    wtr.write_object_start()?;
                }
                BinaryToken::HiddenObject(_) => {
                    wtr.write_hidden_object_start()?;
                }
                BinaryToken::Array(_) => {
                    wtr.write_array_start()?;
                }
                BinaryToken::End(_x) => {
                    wtr.write_end()?;
                }
                BinaryToken::Bool(x) => wtr.write_bool(*x)?,
                BinaryToken::U32(x) => wtr.write_u32(*x)?,
                BinaryToken::U64(x) => wtr.write_u64(*x)?,
                BinaryToken::I32(x) => {
                    if known_number {
                        wtr.write_i32(*x)?;
                        known_number = false;
                    } else if known_date {
                        if let Some(date) = Hoi4Date::from_binary(*x) {
                            wtr.write_date(date.game_fmt())?;
                        } else if self.on_failed_resolve != FailedResolveStrategy::Error {
                            wtr.write_i32(*x)?;
                        } else {
                            return Err(Hoi4Error::new(Hoi4ErrorKind::InvalidDate(*x)));
                        }
                        known_date = false;
                    } else if let Some(date) = Hoi4Date::from_binary_heuristic(*x) {
                        wtr.write_date(date.game_fmt())?;
                    } else {
                        wtr.write_i32(*x)?;
                    }
                }
                BinaryToken::Quoted(x) => {
                    if wtr.expecting_key() {
                        wtr.write_unquoted(x.as_bytes())?;
                    } else {
                        wtr.write_quoted(x.as_bytes())?;
                    }
                }
                BinaryToken::Unquoted(x) => {
                    wtr.write_unquoted(x.as_bytes())?;
                }
                BinaryToken::F32(x) => wtr.write_f32(flavor.visit_f32(*x))?,
                BinaryToken::F64(x) => wtr.write_f64(flavor.visit_f64(*x))?,
                BinaryToken::Token(x) => match resolver.resolve(*x) {
                    Some(id) if self.rewrite && id == "is_ironman" && wtr.expecting_key() => {
                        let skip = tokens
                            .get(token_idx + 1)
                            .map(|next_token| match next_token {
                                BinaryToken::Object(end) => end + 1,
                                BinaryToken::Array(end) => end + 1,
                                _ => token_idx + 2,
                            })
                            .unwrap_or(token_idx + 1);

                        token_idx = skip;
                        continue;
                    }
                    Some(id) => {
                        known_number = id.ends_with("seed");
                        known_date = id == "date";
                        wtr.write_unquoted(id.as_bytes())?;
                    }
                    None => {
                        unknown_tokens.insert(*x);
                        match self.on_failed_resolve {
                            FailedResolveStrategy::Error => {
                                return Err(Hoi4ErrorKind::UnknownToken { token_id: *x }.into());
                            }
                            FailedResolveStrategy::Ignore if wtr.expecting_key() => {
                                let skip = tokens
                                    .get(token_idx + 1)
                                    .map(|next_token| match next_token {
                                        BinaryToken::Object(end) => end + 1,
                                        BinaryToken::Array(end) => end + 1,
                                        _ => token_idx + 2,
                                    })
                                    .unwrap_or(token_idx + 1);

                                token_idx = skip;
                                continue;
                            }
                            _ => {
                                write!(wtr, "__unknown_0x{:x}", x)?;
                            }
                        }
                    }
                },
                BinaryToken::Rgb(color) => {
                    wtr.write_header(b"rgb")?;
                    wtr.write_array_start()?;
                    wtr.write_u32(color.r)?;
                    wtr.write_u32(color.g)?;
                    wtr.write_u32(color.b)?;
                    wtr.write_end()?;
                }
            }

            token_idx += 1;
        }

        wtr.inner().push(b'\n');
        Ok(())
    }

    /// Given one of the accepted inputs, this will return the save id line (if present in the input)
    /// with the gamestate data decoded from binary to plain text.
    pub fn melt_with_tokens<Q>(
        &self,
        mut data: &[u8],
        resolver: &Q,
    ) -> Result<(Vec<u8>, HashSet<u16>), Hoi4Error>
    where
        Q: TokenResolver,
    {
        let mut result = Vec::with_capacity(data.len());
        result.extend_from_slice(b"HOI4txt\n");
        let header = b"HOI4bin";
        if data.get(..header.len()).map_or(false, |x| x == header) {
            data = &data[header.len()..];
        }

        let mut unknown_tokens = HashSet::new();
        self.convert(data, &mut result, &mut unknown_tokens, resolver)?;
        Ok((result, unknown_tokens))
    }

    /// Given one of the accepted inputs, this will return the save id line (if present in the input)
    /// with the gamestate data decoded from binary to plain text.
    pub fn melt(&self, data: &[u8]) -> Result<(Vec<u8>, HashSet<u16>), Hoi4Error> {
        self.melt_with_tokens(data, &TokenLookup)
    }
}
