//! JSON reading with the reference's strict configuration.
//!
//! Every model, blockstate and pack file is read through a reader set to
//! `Strictness.STRICT`, so the leniencies the underlying library allows
//! elsewhere are rejections here: no comments, no single quotes, no trailing
//! commas, no unquoted keys, no `NaN`.
//!
//! Two details drive the representation. Strings keep UTF-16 code units rather
//! than Rust text, because an escape may produce a lone surrogate that the
//! reference accepts and stores. Numbers keep the literal as written, because
//! the reference defers conversion until a caller asks for a specific type.

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    /// The literal `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, kept as the literal text that produced it.
    Number(String),
    /// A string as UTF-16 code units, which may include lone surrogates.
    String(Vec<u16>),
    /// An array, in order.
    Array(Vec<JsonValue>),
    /// An object, in insertion order.
    Object(JsonObject),
}

/// An object's fields, in the order they were first seen.
///
/// A repeated key keeps the slot it already had and takes the newer value,
/// which is what the reference's map does; moving it to the end would reorder
/// the document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsonObject {
    entries: Vec<(Vec<u16>, JsonValue)>,
}

impl JsonObject {
    /// Insert or replace one field.
    pub fn insert(&mut self, key: Vec<u16>, value: JsonValue) {
        if let Some(slot) = self.entries.iter_mut().find(|(name, _)| *name == key) {
            slot.1 = value;
            return;
        }
        self.entries.push((key, value));
    }

    /// Look one field up by its UTF-16 key.
    pub fn get(&self, key: &[u16]) -> Option<&JsonValue> {
        self.entries
            .iter()
            .find(|(name, _)| name.as_slice() == key)
            .map(|(_, value)| value)
    }

    /// Look one field up by Rust text.
    pub fn get_str(&self, key: &str) -> Option<&JsonValue> {
        let encoded: Vec<u16> = key.encode_utf16().collect();
        self.get(&encoded)
    }

    /// Fields in order.
    pub fn entries(&self) -> impl Iterator<Item = (&[u16], &JsonValue)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value))
    }

    /// Number of fields.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the object has no fields.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Why a document was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// Byte offset where parsing stopped.
    pub offset: usize,
    /// What the reader expected instead.
    pub reason: &'static str,
}

/// Parse one complete document, rejecting anything after it.
pub fn parse(text: &str) -> Result<JsonValue, JsonError> {
    let mut reader = Reader {
        bytes: text.as_bytes(),
        offset: 0,
    };
    reader.skip_whitespace();
    let value = reader.value()?;
    reader.skip_whitespace();
    if reader.offset != reader.bytes.len() {
        return Err(reader.fail("trailing content"));
    }
    Ok(value)
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn fail(&self, reason: &'static str) -> JsonError {
        JsonError {
            offset: self.offset,
            reason,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }

    /// Strict JSON allows only these four as whitespace.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.offset += 1;
        }
    }

    fn expect(&mut self, byte: u8, reason: &'static str) -> Result<(), JsonError> {
        if self.peek() == Some(byte) {
            self.offset += 1;
            return Ok(());
        }
        Err(self.fail(reason))
    }

    fn literal(&mut self, word: &[u8], reason: &'static str) -> Result<(), JsonError> {
        if self.bytes[self.offset..].starts_with(word) {
            self.offset += word.len();
            return Ok(());
        }
        Err(self.fail(reason))
    }

    fn value(&mut self) -> Result<JsonValue, JsonError> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(JsonValue::String(self.string()?)),
            Some(b't') => {
                self.literal(b"true", "expected true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false", "expected false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.literal(b"null", "expected null")?;
                Ok(JsonValue::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.fail("expected a value")),
        }
    }

    fn object(&mut self) -> Result<JsonValue, JsonError> {
        self.expect(b'{', "expected {")?;
        let mut object = JsonObject::default();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.offset += 1;
            return Ok(JsonValue::Object(object));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.fail("expected a quoted key"));
            }
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':', "expected :")?;
            self.skip_whitespace();
            let value = self.value()?;
            object.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    return Ok(JsonValue::Object(object));
                }
                _ => return Err(self.fail("expected , or }")),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue, JsonError> {
        self.expect(b'[', "expected [")?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.offset += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.fail("expected , or ]")),
            }
        }
    }

    /// Read one string into UTF-16 code units.
    ///
    /// Raw bytes are already valid UTF-8 because the caller hands over decoded
    /// text; `\u` escapes are taken at face value, so an unpaired surrogate
    /// survives exactly as the reference stores it.
    fn string(&mut self) -> Result<Vec<u16>, JsonError> {
        self.expect(b'"', "expected a string")?;
        let mut units = Vec::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.fail("unterminated string"));
            };
            match byte {
                b'"' => {
                    self.offset += 1;
                    return Ok(units);
                }
                b'\\' => {
                    self.offset += 1;
                    let Some(escape) = self.peek() else {
                        return Err(self.fail("unterminated escape"));
                    };
                    self.offset += 1;
                    match escape {
                        b'"' => units.push(u16::from(b'"')),
                        b'\\' => units.push(u16::from(b'\\')),
                        b'/' => units.push(u16::from(b'/')),
                        b'b' => units.push(0x08),
                        b'f' => units.push(0x0C),
                        b'n' => units.push(u16::from(b'\n')),
                        b'r' => units.push(u16::from(b'\r')),
                        b't' => units.push(u16::from(b'\t')),
                        b'u' => units.push(self.hex4()?),
                        _ => return Err(self.fail("unknown escape")),
                    }
                }
                // Strict JSON forbids raw control characters inside a string.
                0x00..=0x1F => return Err(self.fail("control character in string")),
                _ => {
                    let rest = &self.bytes[self.offset..];
                    let text = core::str::from_utf8(rest).map_err(|_| self.fail("invalid text"))?;
                    let character = text
                        .chars()
                        .next()
                        .ok_or_else(|| self.fail("invalid text"))?;
                    let mut buffer = [0_u16; 2];
                    units.extend_from_slice(character.encode_utf16(&mut buffer));
                    self.offset += character.len_utf8();
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u16, JsonError> {
        let digits = self
            .bytes
            .get(self.offset..self.offset + 4)
            .ok_or_else(|| self.fail("short unicode escape"))?;
        let mut value = 0_u16;
        for digit in digits {
            let nibble = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                b'A'..=b'F' => digit - b'A' + 10,
                _ => return Err(self.fail("bad unicode escape")),
            };
            value = value * 16 + u16::from(nibble);
        }
        self.offset += 4;
        Ok(value)
    }

    /// Read one number, keeping the literal text.
    ///
    /// The grammar is JSON's: an optional sign, an integer part with no leading
    /// zero, an optional fraction with at least one digit, and an optional
    /// exponent. `+1`, `01`, `.5` and `1.` are all rejected.
    fn number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.offset;
        if self.peek() == Some(b'-') {
            self.offset += 1;
        }
        match self.peek() {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.offset += 1;
                }
            }
            _ => return Err(self.fail("expected a digit")),
        }
        if self.peek() == Some(b'.') {
            self.offset += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("expected a digit after ."));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("expected a digit in the exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
        }
        let literal = core::str::from_utf8(&self.bytes[start..self.offset])
            .map_err(|_| self.fail("invalid number"))?;
        Ok(JsonValue::Number(literal.to_owned()))
    }
}

/// A number literal split into sign, digits and a power of ten.
struct DecimalLiteral {
    negative: bool,
    /// Significant digits with the decimal point removed.
    digits: String,
    /// How many places the point moves right; negative drops digits.
    exponent: i64,
}

impl DecimalLiteral {
    /// Split a literal that already parsed as JSON, so the grammar is known good.
    fn split(literal: &str) -> Option<Self> {
        let (negative, rest) = match literal.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, literal),
        };
        let (mantissa, exponent_text) = match rest.find(['e', 'E']) {
            Some(index) => (&rest[..index], Some(&rest[index + 1..])),
            None => (rest, None),
        };
        let (integer, fraction) = match mantissa.find('.') {
            Some(index) => (&mantissa[..index], &mantissa[index + 1..]),
            None => (mantissa, ""),
        };
        let written_exponent = match exponent_text {
            Some(text) => text.strip_prefix('+').unwrap_or(text).parse::<i64>().ok()?,
            None => 0,
        };
        Some(Self {
            negative,
            digits: format!("{integer}{fraction}"),
            // Every fraction digit already moved the point one place left.
            exponent: written_exponent - fraction.len() as i64,
        })
    }

    /// The integer value truncated toward zero, modulo 2^64.
    ///
    /// The reference reaches the same place by three different routes - a
    /// direct int parse, a long parse narrowed by a cast, or a big decimal
    /// whose low bits are taken - and all three amount to this.
    fn truncated_low_bits(&self) -> u64 {
        let mut magnitude: u64 = 0;
        let kept: &str = if self.exponent >= 0 {
            &self.digits
        } else {
            // A negative exponent drops that many digits from the right, which
            // is what truncating toward zero does to the written value.
            let keep = self.digits.len() as i64 + self.exponent;
            if keep <= 0 {
                return 0;
            }
            &self.digits[..keep as usize]
        };
        for digit in kept.bytes() {
            magnitude = magnitude
                .wrapping_mul(10)
                .wrapping_add(u64::from(digit - b'0'));
        }
        for _ in 0..self.exponent.max(0) {
            magnitude = magnitude.wrapping_mul(10);
        }
        if self.negative {
            magnitude.wrapping_neg()
        } else {
            magnitude
        }
    }
}

impl JsonValue {
    /// Java `JsonPrimitive.getAsInt` for a parsed number.
    ///
    /// The value is truncated toward zero and then narrowed to 32 bits, so a
    /// literal past the range wraps rather than saturating.
    pub fn as_i32(&self) -> Option<i32> {
        let Self::Number(literal) = self else {
            return None;
        };
        let split = DecimalLiteral::split(literal)?;
        Some(split.truncated_low_bits() as i32)
    }

    /// Java `JsonPrimitive.getAsLong` for a parsed number.
    pub fn as_i64(&self) -> Option<i64> {
        let Self::Number(literal) = self else {
            return None;
        };
        let split = DecimalLiteral::split(literal)?;
        Some(split.truncated_low_bits() as i64)
    }

    /// Java `JsonPrimitive.getAsFloat`, which overflows to infinity.
    pub fn as_f32(&self) -> Option<f32> {
        let Self::Number(literal) = self else {
            return None;
        };
        literal.parse::<f32>().ok()
    }

    /// Java `JsonPrimitive.getAsDouble`, which overflows to infinity.
    pub fn as_f64(&self) -> Option<f64> {
        let Self::Number(literal) = self else {
            return None;
        };
        literal.parse::<f64>().ok()
    }
}

/// Typed field access mirroring the reference's JSON helper.
///
/// The reference distinguishes a missing field, a field of the wrong shape and
/// a field it can convert, and the model parser depends on all three, so these
/// return `None` where the reference would raise.
impl JsonObject {
    /// A field that must be an object.
    pub fn object(&self, key: &str) -> Option<&JsonObject> {
        match self.get_str(key)? {
            JsonValue::Object(object) => Some(object),
            _ => None,
        }
    }

    /// A field that must be an array.
    pub fn array(&self, key: &str) -> Option<&[JsonValue]> {
        match self.get_str(key)? {
            JsonValue::Array(items) => Some(items),
            _ => None,
        }
    }

    /// A field that must be a string, decoded to Rust text.
    ///
    /// A key holding an unpaired surrogate has no Rust text, and the reference
    /// has no such name either, so it reads as absent.
    pub fn text(&self, key: &str) -> Option<String> {
        match self.get_str(key)? {
            JsonValue::String(units) => String::from_utf16(units).ok(),
            _ => None,
        }
    }

    /// A field that must be a number, read as `f32` the way the reference does.
    pub fn f32(&self, key: &str) -> Option<f32> {
        self.get_str(key)?.as_f32()
    }

    /// A field that must be a number, read as `i32`.
    pub fn i32(&self, key: &str) -> Option<i32> {
        self.get_str(key)?.as_i32()
    }

    /// A field that must be a boolean.
    ///
    /// Only a real boolean counts; the reference's own guards check this
    /// before converting, so a number in a boolean slot is a rejection rather
    /// than a coercion.
    pub fn bool(&self, key: &str) -> Option<bool> {
        match self.get_str(key)? {
            JsonValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Whether the field is present at all.
    pub fn has(&self, key: &str) -> bool {
        self.get_str(key).is_some()
    }
}
