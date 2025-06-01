use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TemplatedInput<T>(pub T);

impl<T> FromStr for TemplatedInput<T>
where
    T: FromStr,
    T::Err: Into<anyhow::Error>,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TemplatedInput(s.parse().map_err(Into::into)?))
    }
}

impl<'de, T> Visitor<'de> for TemplatedInput<T>
where
    T: FromStr,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let need = match std::any::type_name::<T>() {
            "std::str" | "std::str::String" => "string",
            "std::u64" => "unsigned integer",
            "std::i64" => "signed integer",
            "std::bool" => "boolean",
            _ => "unknown type",
        };
        formatter.write_str(&format!("expecting a string that parses as {}", need))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.parse()
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self))?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "signed_integer")]
    SignedInteger,
    #[serde(rename = "select")]
    Select(Vec<SelectOption>),
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub name: String,
    pub value: Input,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Input {
    #[serde(rename = "integer")]
    Integer(u64),
    #[serde(rename = "signed_integer")]
    SignedInteger(i64),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "null")]
    Null,
}

impl ToString for Input {
    fn to_string(&self) -> String {
        match self {
            Input::Integer(x) => x.to_string(),
            Input::SignedInteger(x) => x.to_string(),
            Input::String(x) => x.to_string(),
            Input::Boolean(x) => x.to_string(),
            Input::Null => "null".to_string(),
        }
    }
}
