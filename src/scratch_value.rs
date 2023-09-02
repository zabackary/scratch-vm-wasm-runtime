use std::{convert::TryFrom, ops};

use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub enum ScratchValue {
    /// Represented as 0/1 in the value
    Boolean(bool),
    /// Represented as a pointer
    String(String),
    /// Represented as a 64-bit float (same as JS)
    Number(f64),
}

impl ScratchValue {
    pub const EMPTY: ScratchValue = ScratchValue::String(String::new());
    pub const EMPTY_REF: &ScratchValue = &Self::EMPTY;
}

impl Into<bool> for ScratchValue {
    fn into(self) -> bool {
        match self {
            Self::Boolean(value) => value,
            Self::Number(value) => value != 0f64,
            Self::String(value) => value != "" && value != "false",
        }
    }
}

impl Into<String> for ScratchValue {
    fn into(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Boolean(true) => "true".into(),
            Self::Boolean(false) => "false".into(),
            // May be source of incompatibilities; 1.0.to_string() != "1"
            Self::Number(value) => value.to_string(),
        }
    }
}

impl Into<f64> for ScratchValue {
    fn into(self) -> f64 {
        match self {
            Self::Number(value) => value,
            Self::Boolean(true) => 1f64,
            Self::Boolean(false) => 0f64,
            Self::String(value) => value.parse().unwrap_or(0f64),
        }
    }
}

impl From<bool> for ScratchValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for ScratchValue {
    fn from(value: String) -> Self {
        // Do some fancy conversion stuff
        if value == "true" {
            Self::Boolean(true)
        } else if value == "false" {
            Self::Boolean(false)
        } else if let Ok(value) = value.parse::<f64>() {
            Self::Number(value)
        } else {
            Self::String(value)
        }
    }
}

impl From<f64> for ScratchValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl ops::Add for ScratchValue {
    type Output = ScratchValue;
    fn add(self, rhs: ScratchValue) -> Self::Output {
        Self::Number(Into::<f64>::into(self) + Into::<f64>::into(rhs))
    }
}

impl ops::Sub for ScratchValue {
    type Output = ScratchValue;
    fn sub(self, rhs: ScratchValue) -> Self::Output {
        Self::Number(Into::<f64>::into(self) - Into::<f64>::into(rhs))
    }
}

impl ops::Mul for ScratchValue {
    type Output = ScratchValue;
    fn mul(self, rhs: ScratchValue) -> Self::Output {
        Self::Number(Into::<f64>::into(self) * Into::<f64>::into(rhs))
    }
}

impl ops::Div for ScratchValue {
    type Output = ScratchValue;
    fn div(self, rhs: ScratchValue) -> Self::Output {
        Self::Number(Into::<f64>::into(self) / Into::<f64>::into(rhs))
    }
}

impl TryFrom<JsValue> for ScratchValue {
    type Error = &'static str;
    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        if let Some(value) = value.as_bool() {
            Ok(Self::Boolean(value))
        } else if let Some(value) = value.as_f64() {
            Ok(Self::Number(value))
        } else if let Some(value) = value.as_string() {
            Ok(Self::String(value))
        } else {
            Err("Failed to parse JsValue into ScratchValue (not a primitive type?)")
        }
    }
}

impl Into<JsValue> for ScratchValue {
    fn into(self) -> JsValue {
        match self {
            Self::Boolean(value) => JsValue::from_bool(value),
            Self::String(value) => JsValue::from_str(&value),
            Self::Number(value) => JsValue::from_f64(value),
        }
    }
}
