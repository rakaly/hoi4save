use crate::{flavor::Hoi4Flavor, Hoi4Date, Hoi4Error, Hoi4ErrorKind};
use jomini::{
    binary::{self, BinaryFlavor, FailedResolveStrategy, TokenReader, TokenResolver},
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
    let mut unknown_tokens = HashSet::new();
    let mut reader = TokenReader::new(input);
    let flavor = Hoi4Flavor;

    let mut wtr = TextWriterBuilder::new()
        .indent_char(b'\t')
        .indent_factor(1)
        .from_writer(output);

    let mut known_number = false;
    let mut known_date = false;
    let mut quoted_buffer_enabled = false;
    let mut quoted_buffer: Vec<u8> = Vec::new();
    while let Some(token) = reader.next()? {
        if quoted_buffer_enabled {
            if matches!(token, binary::Token::Equal) {
                wtr.write_unquoted(&quoted_buffer)?;
            } else {
                wtr.write_quoted(&quoted_buffer)?;
            }
            quoted_buffer.clear();
            quoted_buffer_enabled = false;
        }

        match token {
            binary::Token::Open => wtr.write_start()?,
            binary::Token::Close => wtr.write_end()?,
            binary::Token::I32(x) => {
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
            binary::Token::Quoted(x) => {
                if wtr.at_unknown_start() {
                    quoted_buffer_enabled = true;
                    quoted_buffer.extend_from_slice(x.as_bytes());
                } else if wtr.expecting_key() {
                    wtr.write_unquoted(x.as_bytes())?;
                } else {
                    wtr.write_quoted(x.as_bytes())?;
                }
            }
            binary::Token::Unquoted(x) => {
                wtr.write_unquoted(x.as_bytes())?;
            }
            binary::Token::F32(x) => wtr.write_f32(flavor.visit_f32(x))?,
            binary::Token::F64(x) => wtr.write_f64(flavor.visit_f64(x))?,
            binary::Token::Id(0) | binary::Token::Id(0xFFFF) => {
                // Skip null tokens - they appear as padding in newer saves
                if wtr.expecting_key() {
                    // When Id(0) is a key, skip the entire key=value pair
                    let mut next = reader.read()?;
                    if matches!(next, binary::Token::Equal) {
                        next = reader.read()?;
                    }
                    if matches!(next, binary::Token::Open) {
                        reader.skip_container()?;
                    }
                }
                // When Id(0) is a value or array element, just skip it
                continue;
            }
            binary::Token::Id(x) => match resolver.resolve(x) {
                Some(id) => {
                    if !options.verbatim
                        && matches!(id, "is_ironman" | "ironman")
                        && wtr.expecting_key()
                    {
                        let mut next = reader.read()?;
                        if matches!(next, binary::Token::Equal) {
                            next = reader.read()?;
                        }

                        if matches!(next, binary::Token::Open) {
                            reader.skip_container()?;
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
                        return Err(Hoi4ErrorKind::UnknownToken { token_id: x }.into());
                    }
                    FailedResolveStrategy::Ignore if wtr.expecting_key() => {
                        let mut next = reader.read()?;
                        if matches!(next, binary::Token::Equal) {
                            next = reader.read()?;
                        }

                        if matches!(next, binary::Token::Open) {
                            reader.skip_container()?;
                        }
                    }
                    _ => {
                        unknown_tokens.insert(x);
                        write!(wtr, "__unknown_0x{:x}", x)?;
                    }
                },
            },
            binary::Token::Equal => wtr.write_operator(jomini::text::Operator::Equal)?,
            binary::Token::U32(x) => wtr.write_u32(x)?,
            binary::Token::U64(x) => wtr.write_u64(x)?,
            binary::Token::Bool(x) => wtr.write_bool(x)?,
            binary::Token::Rgb(x) => wtr.write_rgb(&x)?,
            binary::Token::I64(x) => wtr.write_i64(x)?,
        }
    }

    Ok(MeltedDocument { unknown_tokens })
}
