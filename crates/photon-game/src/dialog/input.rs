//! The inputs of a dialog (`net.minecraft.server.dialog.input`): what the server asks for and the
//! values the buttons send.

use photon_core::nbt::{Field, FieldValue};

use super::{get, text, width};
use crate::text::Component;

/// `Input`: a name the templates and the payload use, and the control.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub key: String,
    pub control: Control,
}

/// `InputControl`.
#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Boolean(BooleanInput),
    SingleOption(SingleOptionInput),
    NumberRange(NumberRangeInput),
    Text(TextInput),
}

/// `BooleanInput`.
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanInput {
    pub label: Component,
    pub initial: bool,
    pub on_true: String,
    pub on_false: String,
}

/// `SingleOptionInput.Entry`.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionEntry {
    pub id: String,
    pub display: Option<Component>,
    pub initial: bool,
}

/// `SingleOptionInput`.
#[derive(Debug, Clone, PartialEq)]
pub struct SingleOptionInput {
    pub width: i32,
    pub entries: Vec<OptionEntry>,
    pub label: Component,
    pub label_visible: bool,
}

/// `NumberRangeInput.RangeInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeInfo {
    pub start: f32,
    pub end: f32,
    pub initial: Option<f32>,
    pub step: Option<f32>,
}

/// `NumberRangeInput`.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberRangeInput {
    pub width: i32,
    pub label: Component,
    pub label_format: String,
    pub range: RangeInfo,
}

/// `TextInput.MultilineOptions`.
#[derive(Debug, Clone, PartialEq)]
pub struct Multiline {
    pub max_lines: Option<i32>,
    pub height: Option<i32>,
}

/// `TextInput`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextInput {
    pub width: i32,
    pub label: Component,
    pub label_visible: bool,
    pub initial: String,
    pub max_length: i32,
    pub multiline: Option<Multiline>,
}

/// What an input holds when a button is used (`Action.ValueGetter`): its text for a template, and
/// its tag for a payload.
#[derive(Debug, Clone, PartialEq)]
pub struct InputValue {
    pub key: String,
    pub substitution: String,
    pub tag: FieldValue,
}

fn flag(fields: &[Field], key: &str, default: bool) -> Option<bool> {
    match get(fields, key) {
        None => Some(default),
        Some(FieldValue::Byte(value)) => Some(*value != 0),
        Some(_) => None,
    }
}

/// A number tag as a float (`Codec.FLOAT` takes any of them).
fn number(value: &FieldValue) -> Option<f32> {
    match value {
        FieldValue::Byte(v) => Some(f32::from(*v)),
        FieldValue::Short(v) => Some(f32::from(*v)),
        FieldValue::Int(v) => Some(*v as f32),
        FieldValue::Long(v) => Some(*v as f32),
        FieldValue::Float(v) => Some(*v),
        FieldValue::Double(v) => Some(*v as f32),
        _ => None,
    }
}

fn optional_number(fields: &[Field], key: &str) -> Option<Option<f32>> {
    match get(fields, key) {
        None => Some(None),
        Some(value) => number(value).map(Some),
    }
}

fn label(fields: &[Field]) -> Option<Component> {
    Component::from_nbt(get(fields, "label")?)
}

fn positive_int(fields: &[Field], key: &str, default: i32) -> Option<i32> {
    match get(fields, key) {
        None => Some(default),
        Some(FieldValue::Int(value)) if *value >= 1 => Some(*value),
        Some(_) => None,
    }
}

impl Input {
    /// `Input.CODEC`: the key and the control the `type` names.
    pub fn from_nbt(value: &FieldValue) -> Option<Self> {
        let FieldValue::Compound(fields) = value else {
            return None;
        };
        let key = text(fields, "key")?;
        if !super::template::is_valid_variable_name(key) {
            return None;
        }
        let control = match super::strip_namespace(text(fields, "type")?) {
            "boolean" => Control::Boolean(BooleanInput::from_fields(fields)?),
            "single_option" => Control::SingleOption(SingleOptionInput::from_fields(fields)?),
            "number_range" => Control::NumberRange(NumberRangeInput::from_fields(fields)?),
            "text" => Control::Text(TextInput::from_fields(fields)?),
            _ => return None,
        };
        Some(Self {
            key: key.to_owned(),
            control,
        })
    }
}

impl BooleanInput {
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let string = |key: &str, default: &str| match get(fields, key) {
            None => Some(default.to_owned()),
            Some(FieldValue::Str(value)) => Some(value.clone()),
            Some(_) => None,
        };
        Some(Self {
            label: label(fields)?,
            initial: flag(fields, "initial", false)?,
            on_true: string("on_true", "true")?,
            on_false: string("on_false", "false")?,
        })
    }
}

impl OptionEntry {
    /// `Entry.CODEC`: the record, or a bare id.
    fn from_nbt(value: &FieldValue) -> Option<Self> {
        match value {
            FieldValue::Str(id) => Some(Self {
                id: id.clone(),
                display: None,
                initial: false,
            }),
            FieldValue::Compound(fields) => Some(Self {
                id: text(fields, "id")?.to_owned(),
                display: get(fields, "display").and_then(Component::from_nbt),
                initial: flag(fields, "initial", false)?,
            }),
            _ => None,
        }
    }
}

impl SingleOptionInput {
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let Some(FieldValue::List(items)) = get(fields, "options") else {
            return None;
        };
        let entries: Vec<OptionEntry> = items
            .iter()
            .map(OptionEntry::from_nbt)
            .collect::<Option<_>>()?;
        // `ExtraCodecs.nonEmptyList`, and at most one entry may be the first shown.
        if entries.is_empty() || entries.iter().filter(|entry| entry.initial).count() > 1 {
            return None;
        }
        Some(Self {
            width: width(fields, "width", 200, 1024)?,
            entries,
            label: label(fields)?,
            label_visible: flag(fields, "label_visible", true)?,
        })
    }

    /// `SingleOptionInput.initial`, else the first entry (`SingleOptionHandler`).
    pub fn initial_index(&self) -> usize {
        self.entries
            .iter()
            .position(|entry| entry.initial)
            .unwrap_or(0)
    }
}

impl NumberRangeInput {
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let range = RangeInfo {
            start: number(get(fields, "start")?)?,
            end: number(get(fields, "end")?)?,
            initial: optional_number(fields, "initial")?,
            step: optional_number(fields, "step")?,
        };
        // `ExtraCodecs.POSITIVE_FLOAT`, and an initial value inside the range.
        if range.step.is_some_and(|step| step <= 0.0 || step.is_nan()) {
            return None;
        }
        if let Some(initial) = range.initial {
            let (low, high) = (range.start.min(range.end), range.start.max(range.end));
            if initial < low || initial > high {
                return None;
            }
        }
        Some(Self {
            width: width(fields, "width", 200, 1024)?,
            label: label(fields)?,
            label_format: match get(fields, "label_format") {
                None => "options.generic_value".to_owned(),
                Some(FieldValue::Str(format)) => format.clone(),
                Some(_) => return None,
            },
            range,
        })
    }
}

impl TextInput {
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let initial = match get(fields, "initial") {
            None => String::new(),
            Some(FieldValue::Str(initial)) => initial.clone(),
            Some(_) => return None,
        };
        let max_length = positive_int(fields, "max_length", 32)?;
        // The default text must fit (`String.length` counts UTF-16 units).
        if initial.encode_utf16().count() as i64 > i64::from(max_length) {
            return None;
        }
        Some(Self {
            width: width(fields, "width", 200, 1024)?,
            label: label(fields)?,
            label_visible: flag(fields, "label_visible", true)?,
            initial,
            max_length,
            multiline: match get(fields, "multiline") {
                None => None,
                Some(FieldValue::Compound(options)) => Some(Multiline {
                    max_lines: match get(options, "max_lines") {
                        None => None,
                        Some(_) => Some(positive_int(options, "max_lines", 1)?),
                    },
                    height: match get(options, "height") {
                        None => None,
                        Some(_) => Some(width(options, "height", 1, 512)?),
                    },
                }),
                Some(_) => return None,
            },
        })
    }
}

/// `Math.round(float)`: halves go up.
fn java_round(value: f32) -> i32 {
    (f64::from(value) + 0.5).floor() as i32
}

impl RangeInfo {
    fn initial_scaled(&self) -> f32 {
        self.initial.unwrap_or((self.start + self.end) / 2.0)
    }

    /// `scaledValueToSlider`.
    fn to_slider(&self, value: f32) -> f32 {
        if self.start == self.end {
            0.5
        } else {
            (value - self.start) / (self.end - self.start)
        }
    }

    /// `initialSliderValue`: where the handle starts, 0 to 1.
    pub fn initial_slider(&self) -> f32 {
        self.to_slider(self.initial_scaled())
    }

    /// `computeScaledValue`: the value the handle stands for, on the steps if there are any.
    pub fn scaled(&self, slider: f32) -> f32 {
        let in_range = self.start + slider * (self.end - self.start);
        let Some(step) = self.step else {
            return in_range;
        };
        let initial = self.initial_scaled();
        let steps = java_round((in_range - initial) / step);
        let result = initial + steps as f32 * step;
        let position = self.to_slider(result);
        if (0.0..=1.0).contains(&position) {
            return result;
        }
        let one_less = steps - steps.signum();
        initial + one_less as f32 * step
    }
}

/// `valueToString`: a whole number as an integer, else `Float.toString`.
pub fn range_text(value: f32) -> String {
    if value as i32 as f32 == value {
        (value as i32).to_string()
    } else {
        crate::commands::reader::java_float(value)
    }
}
