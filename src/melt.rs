use crate::{FailedResolveStrategy, Hoi4Date, Hoi4Error, Hoi4ErrorKind, flavor::Hoi4Flavor, tokens::TokenLookup};
use jomini::{BinaryTape, BinaryToken, TokenResolver};
use std::{collections::HashSet, io::Write};

/// Convert a binary gamestate to plaintext
#[derive(Debug)]
pub struct Melter {
    on_failed_resolve: FailedResolveStrategy,
}

impl Default for Melter {
    fn default() -> Self {
        Melter {
            on_failed_resolve: FailedResolveStrategy::Stringify,
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

    fn convert(
        &self,
        input: &[u8],
        writer: &mut Vec<u8>,
        unknown_tokens: &mut HashSet<u16>,
    ) -> Result<(), Hoi4Error> {
        let tape = BinaryTape::parser_flavor(Hoi4Flavor).parse_slice(input)?;
        let mut depth = 0;
        let mut in_objects: Vec<i32> = Vec::new();
        let mut in_object = 1;
        let mut token_idx = 0;
        let mut known_number = false;
        let tokens = tape.tokens();

        while let Some(token) = tokens.get(token_idx) {
            let mut did_change = false;
            if in_object == 1 {
                let depth = match token {
                    BinaryToken::End(_) => depth - 1,
                    _ => depth,
                };

                for _ in 0..depth {
                    writer.push(b' ');
                }
            }

            match token {
                BinaryToken::Object(_) => {
                    did_change = true;
                    writer.extend_from_slice(b"{\r\n");
                    depth += 1;
                    in_objects.push(in_object);
                    in_object = 1;
                }
                BinaryToken::HiddenObject(_) => {
                    did_change = true;
                    depth += 1;
                    in_objects.push(in_object);
                    in_object = 1;
                }
                BinaryToken::Array(_) => {
                    did_change = true;
                    writer.push(b'{');
                    depth += 1;
                    in_objects.push(in_object);
                    in_object = 0;
                }
                BinaryToken::End(x) => {
                    if !matches!(tokens.get(*x), Some(BinaryToken::HiddenObject(_))) {
                        writer.push(b'}');
                    }
                    let obj = in_objects.pop();

                    // The binary parser should already ensure that this will be something, but this is
                    // just a sanity check
                    debug_assert!(obj.is_some());
                    in_object = obj.unwrap_or(1);
                    depth -= 1;
                }
                BinaryToken::Bool(x) => match x {
                    true => writer.extend_from_slice(b"yes"),
                    false => writer.extend_from_slice(b"no"),
                },
                BinaryToken::U32(x) => writer.extend_from_slice(format!("{}", x).as_bytes()),
                BinaryToken::U64(x) => writer.extend_from_slice(format!("{}", x).as_bytes()),
                BinaryToken::I32(x) => {
                    if known_number {
                        writer.extend_from_slice(format!("{}", x).as_bytes());
                        known_number = false;
                    } else if let Some(date) = Hoi4Date::from_binary(*x) {
                        writer.extend_from_slice(date.game_fmt().as_bytes());
                    } else {
                        writer.extend_from_slice(format!("{}", x).as_bytes());
                    }
                }
                BinaryToken::Text(x) => {
                    let data = x.view_data();
                    let end_idx = match data.last() {
                        Some(x) if *x == b'\n' => data.len() - 1,
                        Some(_x) => data.len(),
                        None => data.len(),
                    };
                    if in_object == 1 {
                        writer.extend_from_slice(&data[..end_idx]);
                    } else {
                        writer.push(b'"');
                        writer.extend_from_slice(&data[..end_idx]);
                        writer.push(b'"');
                    }
                }
                BinaryToken::F32_1(x) => write!(writer, "{}", x).map_err(Hoi4ErrorKind::IoErr)?,
                BinaryToken::F32_2(x) => write!(writer, "{}", x).map_err(Hoi4ErrorKind::IoErr)?,
                BinaryToken::Token(x) => match TokenLookup.resolve(*x) {
                    Some(id) if id == "is_ironman" && in_object == 1 => {
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
                        known_number = in_object == 1 && id.ends_with("seed");
                        writer.extend_from_slice(&id.as_bytes())
                    }
                    None => {
                        unknown_tokens.insert(*x);
                        match self.on_failed_resolve {
                            FailedResolveStrategy::Error => {
                                return Err(Hoi4ErrorKind::UnknownToken { token_id: *x }.into());
                            }
                            FailedResolveStrategy::Ignore if in_object == 1 => {
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
                                let unknown = format!("__unknown_0x{:x}", x);
                                writer.extend_from_slice(unknown.as_bytes());
                            }
                        }
                    }
                },
                BinaryToken::Rgb(color) => {
                    writer.extend_from_slice(b"rgb {");
                    writer.extend_from_slice(format!("{} ", color.r).as_bytes());
                    writer.extend_from_slice(format!("{} ", color.g).as_bytes());
                    writer.extend_from_slice(format!("{}", color.b).as_bytes());
                    writer.push(b'}');
                }
            }

            if !did_change && in_object == 1 {
                writer.push(b'=');
                in_object = 2;
            } else if in_object == 2 {
                in_object = 1;
                writer.push(b'\r');
                writer.push(b'\n');
            } else if in_object != 1 {
                writer.push(b' ');
            }

            token_idx += 1;
        }

        Ok(())
    }

    /// Given one of the accepted inputs, this will return the save id line (if present in the input)
    /// with the gamestate data decoded from binary to plain text.
    pub fn melt(&self, mut data: &[u8]) -> Result<(Vec<u8>, HashSet<u16>), Hoi4Error> {
        let mut result = Vec::with_capacity(data.len());
        result.extend_from_slice(b"HOI4txt\r\n");
        let header = b"HOI4bin";
        if data.get(..header.len()).map_or(false, |x| x == header) {
            data = &data[header.len()..];
        }

        let mut unknown_tokens = HashSet::new();
        self.convert(data, &mut result, &mut unknown_tokens)?;
        Ok((result, unknown_tokens))
    }
}
