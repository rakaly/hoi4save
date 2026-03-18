use crate::{
    flavor::{Hoi4Flavor, Hoi4Format, Hoi4Token},
    Hoi4Date, Hoi4Error, Hoi4ErrorKind,
};
use jomini::{
    binary::{ng::TokenReader, FailedResolveStrategy, TokenResolver},
    common::PdsDate,
    TextWriterBuilder,
};
use std::{
    collections::HashSet,
    io::{Read, Write},
};

/// Output from melting a binary save to plaintext
#[derive(Debug, Default)]
pub struct MeltedDocument {
    unknown_tokens: HashSet<u16>,
}

impl MeltedDocument {
    pub fn new() -> Self {
        Self::default()
    }

    /// The list of unknown tokens that the provided resolver accumulated
    pub fn unknown_tokens(&self) -> &HashSet<u16> {
        &self.unknown_tokens
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeltOptions {
    verbatim: bool,
    on_failed_resolve: FailedResolveStrategy,
}

impl Default for MeltOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MeltOptions {
    pub fn new() -> Self {
        Self {
            verbatim: false,
            on_failed_resolve: FailedResolveStrategy::Ignore,
        }
    }

    pub fn verbatim(self, verbatim: bool) -> Self {
        MeltOptions { verbatim, ..self }
    }

    pub fn on_failed_resolve(self, on_failed_resolve: FailedResolveStrategy) -> Self {
        MeltOptions {
            on_failed_resolve,
            ..self
        }
    }
}

pub(crate) fn melt<Reader, Writer, Resolver>(
    input: Reader,
    output: Writer,
    resolver: Resolver,
    options: MeltOptions,
) -> Result<MeltedDocument, Hoi4Error>
where
    Reader: Read,
    Writer: Write,
    Resolver: TokenResolver,
{
    let mut reader = TokenReader::new(
        input,
        Hoi4Format::new(&resolver).with_failed_resolve_strategy(options.on_failed_resolve),
    );
    let mut unknown_tokens = HashSet::new();

    let mut wtr = TextWriterBuilder::new()
        .indent_char(b'\t')
        .indent_factor(1)
        .from_writer(output);

    let mut known_number = false;
    let mut known_date = false;
    let mut quoted_buffer_enabled = false;
    let mut quoted_buffer: Vec<u8> = Vec::new();

    while let Some(token) = reader.next_token()? {
        if quoted_buffer_enabled {
            if matches!(token, Hoi4Token::Equal) {
                wtr.write_unquoted(&quoted_buffer)?;
            } else {
                wtr.write_quoted(&quoted_buffer)?;
            }
            quoted_buffer.clear();
            quoted_buffer_enabled = false;
        }

        match token {
            Hoi4Token::Open => wtr.write_start()?,
            Hoi4Token::Close => wtr.write_end()?,
            Hoi4Token::Equal => wtr.write_operator(jomini::text::Operator::Equal)?,
            Hoi4Token::U32(x) => wtr.write_u32(x)?,
            Hoi4Token::U64(x) => wtr.write_u64(x)?,
            Hoi4Token::I32(x) => {
                if known_number {
                    wtr.write_i32(x)?;
                    known_number = false;
                } else if known_date {
                    if let Some(date) = Hoi4Date::from_binary(x) {
                        wtr.write_date(date.game_fmt())?;
                    } else if options.on_failed_resolve != FailedResolveStrategy::Error {
                        wtr.write_i32(x)?;
                    } else {
                        return Err(Hoi4Error::from(Hoi4ErrorKind::InvalidDate(x)));
                    }
                    known_date = false;
                } else if let Some(date) = Hoi4Date::from_binary_heuristic(x) {
                    wtr.write_date(date.game_fmt())?;
                } else {
                    wtr.write_i32(x)?;
                }
            }
            Hoi4Token::I64(x) => wtr.write_i64(x)?,
            Hoi4Token::Bool(x) => wtr.write_bool(x)?,
            Hoi4Token::Unquoted(x) => wtr.write_unquoted(x)?,
            Hoi4Token::Quoted(x) => {
                if wtr.at_unknown_start() {
                    quoted_buffer_enabled = true;
                    quoted_buffer.extend_from_slice(x);
                } else if wtr.expecting_key() {
                    wtr.write_unquoted(x)?;
                } else {
                    wtr.write_quoted(x)?;
                }
            }
            Hoi4Token::LegacyFixedPoint(x) => wtr.write_f32(Hoi4Flavor::decode_legacy_f32(x))?,
            Hoi4Token::ModernFixedPoint(x) => wtr.write_i64(Hoi4Flavor::decode_modern_i64(x))?,
            Hoi4Token::F64(x) => wtr.write_f64(x)?,
            Hoi4Token::Field(x) => match resolver.resolve(x.0) {
                Some(id) => {
                    if !options.verbatim
                        && matches!(id, "is_ironman" | "ironman")
                        && wtr.expecting_key()
                    {
                        if matches!(reader.read_token()?, Hoi4Token::Equal) {
                            reader.skip_value()?;
                        }
                        continue;
                    }

                    known_number =
                        id.ends_with("seed") || matches!(id, "total" | "available" | "locked");
                    known_date = id == "date";
                    wtr.write_unquoted(id.as_bytes())?;
                }
                None => match options.on_failed_resolve {
                    FailedResolveStrategy::Error => {
                        return Err(Hoi4ErrorKind::UnknownToken { token_id: x.0 }.into());
                    }
                    _ => {
                        unknown_tokens.insert(x.0);
                        write!(wtr, "__unknown_0x{:x}", x.0)?;
                    }
                },
            },
        }
    }

    Ok(MeltedDocument { unknown_tokens })
}
