use crate::{PromptCollection, PromptParser, PromptResponse};
use serde::{de::Visitor, Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplatedInput<T> {
    input: String,
    marker: std::marker::PhantomData<T>,
}

impl<T> TemplatedInput<T>
where
    T: FromStr,
    T::Err: Send + Sync + std::error::Error + 'static,
{
    pub fn output(
        &self,
        prompts: PromptCollection,
        responses: Vec<PromptResponse>,
    ) -> Result<T, anyhow::Error> {
        let parser = PromptParser(prompts);
        Ok(parser.template(self.input.clone(), responses)?.parse()?)
    }
}

impl<T> FromStr for TemplatedInput<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            input: s.to_string(),
            marker: Default::default(),
        })
    }
}

impl<'de, T> Visitor<'de> for TemplatedInput<T> {
    type Value = TemplatedInput<T>;

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
        Ok(Self {
            input: v.to_string(),
            marker: Default::default(),
        })
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
