pub(super) fn parse_json(text: &str) -> Json<'_> {
    let text = text.trim();
    match text {
        "true" => Json::Bool(true),
        "false" => Json::Bool(false),
        _ if is_json_number(text) => Json::Number(text),
        _ => json_string(text)
            .or_else(|| bare_word(text))
            .map_or(Json::Other, Json::Text),
    }
}

/// A word without quotes, which the lenient parser reads as a string: it ends at any of these.
fn bare_word(text: &str) -> Option<String> {
    let ends = |c: char| c.is_whitespace() || "/\\;#={}[],:".contains(c);
    let quoted = text.starts_with(['"', '\'']);
    (!text.is_empty() && !quoted && text != "null" && !text.contains(ends)).then(|| text.to_owned())
}

/// A JSON string with the escapes a name can carry.
pub(super) fn json_string(text: &str) -> Option<String> {
    // The lenient reader takes single quotes as well.
    let quote = text.chars().next().filter(|c| matches!(c, '"' | '\''))?;
    let inner = text.strip_prefix(quote)?.strip_suffix(quote)?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            _ if c == quote => return None,
            '\\' => match chars.next()? {
                escaped if escaped == quote => out.push(quote),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                _ => return None,
            },
            _ => out.push(c),
        }
    }
    Some(out)
}

/// A string as the file writes it.
pub(super) fn write_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The JSON number grammar.
pub(super) fn is_json_number(text: &str) -> bool {
    let bytes = text.as_bytes();
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut at = usize::from(bytes.first() == Some(&b'-'));
    match digits(at) {
        0 => return false,
        1 => {}
        _ if bytes[at] == b'0' => return false,
        _ => {}
    }
    at += digits(at);
    if bytes.get(at) == Some(&b'.') {
        let fraction = digits(at + 1);
        if fraction == 0 {
            return false;
        }
        at += 1 + fraction;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        let exponent = digits(at);
        if exponent == 0 {
            return false;
        }
        at += exponent;
    }
    at == bytes.len()
}

/// `Number.doubleValue` of a parsed number.
pub(super) fn double_value(number: &str) -> f64 {
    number.parse().unwrap_or(f64::NAN)
}

/// `Number.intValue` of a parsed number (`LazilyParsedNumber`): an integer
/// as it reads, else the decimal cut towards zero.
pub(super) fn int_value(number: &str) -> i32 {
    if let Ok(value) = number.parse::<i64>() {
        return value as i32;
    }
    double_value(number) as i128 as i32
}

/// `Double.toString`, as `Gson` writes a double.
pub fn java_double(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .to_owned();
    }
    if (1e-3..1e7).contains(&value.abs()) {
        return format!("{value:?}");
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let dot = if mantissa.contains('.') { "" } else { ".0" };
    format!("{mantissa}{dot}E{exponent}")
}

/// A value as the file's JSON holds it (`LenientJsonParser`): a number, a
/// boolean, a string, or anything else, which no option's codec takes.
pub(super) enum Json<'a> {
    Number(&'a str),
    Bool(bool),
    Text(String),
    Other,
}

impl Json<'_> {
    /// The boolean, if that is what the value is.
    pub(super) fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(value) => Some(*value),
            _ => None,
        }
    }
}
