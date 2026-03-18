use jomini::{
    binary::ng::{
        BinaryTokenFormat, BinaryValueFormat, ParserState, PdxVisitor, TokenResult, ValueResult,
    },
    binary::{BinaryFlavor, FailedResolveStrategy, LexemeId, TokenResolver},
    Encoding, Error, Utf8Encoding,
};
use serde::de::Error as _;
use std::borrow::Cow;

const SAVE_VERSION_ID: u16 = 0x349d;

pub struct Hoi4Flavor;

impl Hoi4Flavor {
    #[inline]
    pub(crate) fn decode_legacy_f64(raw: [u8; 4]) -> f64 {
        i32::from_le_bytes(raw) as f64 / 1000.0
    }

    #[inline]
    pub(crate) fn decode_modern_f64(raw: [u8; 8]) -> f64 {
        i64::from_le_bytes(raw) as f64 / 100_000.0
    }

    #[inline]
    pub(crate) fn decode_modern_i64(raw: [u8; 8]) -> i64 {
        i64::from_le_bytes(raw) / 100_000
    }

    #[inline]
    pub(crate) fn decode_legacy_f32(raw: [u8; 4]) -> f32 {
        Self::decode_legacy_f64(raw) as f32
    }

    #[inline]
    pub(crate) fn decode_modern_f32(raw: [u8; 8]) -> f32 {
        Self::decode_modern_f64(raw) as f32
    }

    #[inline]
    pub(crate) fn decode_f64(raw: [u8; 8]) -> f64 {
        let val = i64::from_le_bytes(raw) as f64 / 32768.0;
        (val * 100_000.0).floor() / 100_000.0
    }
}

impl Encoding for Hoi4Flavor {
    fn decode<'a>(&self, data: &'a [u8]) -> std::borrow::Cow<'a, str> {
        Utf8Encoding::decode(data)
    }
}

impl BinaryFlavor for Hoi4Flavor {
    fn visit_f32(&self, data: [u8; 4]) -> f32 {
        Self::decode_legacy_f32(data)
    }

    fn visit_f64(&self, data: [u8; 8]) -> f64 {
        Self::decode_f64(data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hoi4Token<'a> {
    Open,
    Close,
    Equal,
    Field(LexemeId),
    Bool(bool),
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    Quoted(&'a [u8]),
    Unquoted(&'a [u8]),
    LegacyFixedPoint([u8; 4]),
    ModernFixedPoint([u8; 8]),
    F64(f64),
}

pub struct Hoi4Format<R> {
    resolver: R,
    failed_resolve_strategy: FailedResolveStrategy,
    pending_save_version: bool,
    save_version: i32,
}

impl<R: TokenResolver> Hoi4Format<R> {
    pub fn new(resolver: R) -> Self {
        Self {
            resolver,
            failed_resolve_strategy: FailedResolveStrategy::Error,
            pending_save_version: false,
            save_version: 0,
        }
    }

    pub fn with_failed_resolve_strategy(
        mut self,
        failed_resolve_strategy: FailedResolveStrategy,
    ) -> Self {
        self.failed_resolve_strategy = failed_resolve_strategy;
        self
    }

    #[inline]
    fn modern_f32(&self) -> bool {
        self.save_version >= 30
    }

    #[inline]
    fn track_field(&mut self, id: LexemeId) {
        self.pending_save_version = id.0 == SAVE_VERSION_ID;
    }

    #[inline]
    fn visit_i32(&mut self, raw: [u8; 4]) -> i32 {
        let value = i32::from_le_bytes(raw);
        if self.pending_save_version {
            self.save_version = value;
            self.pending_save_version = false;
        }
        value
    }

    #[inline]
    fn resolve_name<'de, V: PdxVisitor<'de>>(
        &self,
        field: LexemeId,
        visitor: V,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        match self.resolver.resolve(field.0) {
            Some(name) => Ok(ValueResult::Value(visitor.visit_str(name)?)),
            None => match self.failed_resolve_strategy {
                FailedResolveStrategy::Error => Err(Error::custom(format!(
                    "unknown field token 0x{:x}",
                    field.0
                ))),
                FailedResolveStrategy::Stringify => Ok(ValueResult::Value(
                    visitor.visit_string(format!("0x{:x}", field.0))?,
                )),
                FailedResolveStrategy::Ignore => Ok(ValueResult::Value(
                    visitor.visit_str("__internal_identifier_ignore")?,
                )),
            },
        }
    }

    fn parse_str_token<'de, V: PdxVisitor<'de>>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };

        if !matches!(id, LexemeId::QUOTED | LexemeId::UNQUOTED) {
            return self.deserialize_any(reader, visitor);
        }

        let Some(data) = cursor.read_len_prefixed() else {
            return Ok(ValueResult::MoreData(visitor));
        };
        cursor.consume();
        let value = match self.decode_scalar(data)? {
            Cow::Borrowed(x) => visitor.visit_str(x)?,
            Cow::Owned(x) => visitor.visit_string(x)?,
        };
        Ok(ValueResult::Value(value))
    }
}

impl<R: TokenResolver> BinaryTokenFormat for Hoi4Format<R> {
    type Token<'a> = Hoi4Token<'a>;

    fn next_token<'a>(
        &mut self,
        reader: &'a mut ParserState,
    ) -> Result<TokenResult<Self::Token<'a>>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(TokenResult::MoreData);
        };

        match id {
            LexemeId::OPEN => {
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::Open))
            }
            LexemeId::CLOSE => {
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::Close))
            }
            LexemeId::EQUAL => {
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::Equal))
            }
            LexemeId::BOOL => {
                let Some(bytes) = cursor.read_bytes::<1>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::Bool(bytes[0] != 0)))
            }
            LexemeId::U32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::U32(u32::from_le_bytes(
                    bytes,
                ))))
            }
            LexemeId::U64 => {
                let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::U64(u64::from_le_bytes(
                    bytes,
                ))))
            }
            LexemeId::I32 => {
                let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::I32(self.visit_i32(bytes))))
            }
            LexemeId::I64 => {
                let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::I64(i64::from_le_bytes(
                    bytes,
                ))))
            }
            LexemeId::QUOTED | LexemeId::UNQUOTED => {
                let Some(data) = cursor.read_len_prefixed() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                if id == LexemeId::QUOTED {
                    Ok(TokenResult::Token(Hoi4Token::Quoted(data)))
                } else {
                    Ok(TokenResult::Token(Hoi4Token::Unquoted(data)))
                }
            }
            LexemeId::F32 => {
                if self.modern_f32() {
                    let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                        return Ok(TokenResult::MoreData);
                    };
                    cursor.consume();
                    Ok(TokenResult::Token(Hoi4Token::ModernFixedPoint(bytes)))
                } else {
                    let Some(bytes) = cursor.read_bytes::<4>().copied() else {
                        return Ok(TokenResult::MoreData);
                    };
                    cursor.consume();
                    Ok(TokenResult::Token(Hoi4Token::LegacyFixedPoint(bytes)))
                }
            }
            LexemeId::F64 => {
                let Some(bytes) = cursor.read_bytes::<8>().copied() else {
                    return Ok(TokenResult::MoreData);
                };
                cursor.consume();
                Ok(TokenResult::Token(Hoi4Token::F64(Hoi4Flavor::decode_f64(
                    bytes,
                ))))
            }
            id => {
                cursor.consume();
                self.track_field(id);
                Ok(TokenResult::Token(Hoi4Token::Field(id)))
            }
        }
    }
}

impl<R: TokenResolver> BinaryValueFormat for Hoi4Format<R> {
    fn decode_scalar<'a>(&self, data: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        Ok(Utf8Encoding::decode(data))
    }

    fn deserialize_str<'de, V: PdxVisitor<'de>>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        self.parse_str_token(reader, visitor)
    }

    fn deserialize_identifier<'de, V: PdxVisitor<'de>>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        let mut cursor = reader.token_cursor();
        let Some(id) = cursor.read_lexeme() else {
            return Ok(ValueResult::MoreData(visitor));
        };

        if matches!(id, LexemeId::QUOTED | LexemeId::UNQUOTED) {
            self.parse_str_token(reader, visitor)
        } else if matches!(
            id,
            LexemeId::OPEN
                | LexemeId::CLOSE
                | LexemeId::EQUAL
                | LexemeId::BOOL
                | LexemeId::U32
                | LexemeId::U64
                | LexemeId::I32
                | LexemeId::I64
                | LexemeId::F32
                | LexemeId::F64
        ) {
            self.deserialize_any(reader, visitor)
        } else {
            cursor.consume();
            self.track_field(id);
            self.resolve_name(id, visitor)
        }
    }

    fn deserialize_any<'de, V: PdxVisitor<'de>>(
        &mut self,
        reader: &mut ParserState,
        visitor: V,
    ) -> Result<ValueResult<V::Value, V>, Error> {
        match self.next_token(reader)? {
            TokenResult::MoreData => Ok(ValueResult::MoreData(visitor)),
            TokenResult::Token(Hoi4Token::Open) => Ok(ValueResult::Open(visitor)),
            TokenResult::Token(Hoi4Token::Close | Hoi4Token::Equal) => {
                Err(Error::custom("unexpected structural token"))
            }
            TokenResult::Token(Hoi4Token::Bool(x)) => {
                Ok(ValueResult::Value(visitor.visit_bool(x)?))
            }
            TokenResult::Token(Hoi4Token::U32(x)) => Ok(ValueResult::Value(visitor.visit_u32(x)?)),
            TokenResult::Token(Hoi4Token::U64(x)) => Ok(ValueResult::Value(visitor.visit_u64(x)?)),
            TokenResult::Token(Hoi4Token::I32(x)) => Ok(ValueResult::Value(visitor.visit_i32(x)?)),
            TokenResult::Token(Hoi4Token::I64(x)) => Ok(ValueResult::Value(visitor.visit_i64(x)?)),
            TokenResult::Token(Hoi4Token::Quoted(data) | Hoi4Token::Unquoted(data)) => {
                let value = match self.decode_scalar(data)? {
                    Cow::Borrowed(x) => visitor.visit_str(x)?,
                    Cow::Owned(x) => visitor.visit_string(x)?,
                };
                Ok(ValueResult::Value(value))
            }
            TokenResult::Token(Hoi4Token::LegacyFixedPoint(raw)) => Ok(ValueResult::Value(
                visitor.visit_f32(Hoi4Flavor::decode_legacy_f32(raw))?,
            )),
            TokenResult::Token(Hoi4Token::ModernFixedPoint(raw)) => Ok(ValueResult::Value(
                visitor.visit_f32(Hoi4Flavor::decode_modern_f32(raw))?,
            )),
            TokenResult::Token(Hoi4Token::F64(x)) => Ok(ValueResult::Value(visitor.visit_f64(x)?)),
            TokenResult::Token(Hoi4Token::Field(id)) => self.resolve_name(id, visitor),
        }
    }
}
