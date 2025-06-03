use crate::{Global, PromptCollection, PromptParser, Responses};
use serde::{de::Visitor, Deserialize, Serialize};
use std::str::FromStr;

//
// see package.rs for some important understanding about this package that I won't repeat here
//

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct TemplatedInput<T> {
    input: String,
    #[serde(skip)]
    marker: std::marker::PhantomData<T>,
}

impl<T> Serialize for TemplatedInput<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.input)
    }
}

impl Default for TemplatedInput<u16> {
    fn default() -> Self {
        TemplatedInput {
            input: "0".into(),
            marker: Default::default(),
        }
    }
}

impl Default for TemplatedInput<u64> {
    fn default() -> Self {
        TemplatedInput {
            input: "0".into(),
            marker: Default::default(),
        }
    }
}

impl Default for TemplatedInput<i64> {
    fn default() -> Self {
        TemplatedInput {
            input: "0".into(),
            marker: Default::default(),
        }
    }
}

impl Default for TemplatedInput<bool> {
    fn default() -> Self {
        TemplatedInput {
            input: "false".into(),
            marker: Default::default(),
        }
    }
}

impl Default for TemplatedInput<String> {
    fn default() -> Self {
        TemplatedInput {
            input: "".into(),
            marker: Default::default(),
        }
    }
}

impl Default for TemplatedInput<&str> {
    fn default() -> Self {
        TemplatedInput {
            input: "".into(),
            marker: Default::default(),
        }
    }
}

impl<T> TemplatedInput<T>
where
    T: FromStr,
    T::Err: Send + Sync + std::error::Error + 'static,
{
    pub fn output<'a>(
        &self,
        globals: &Global,
        prompts: &PromptCollection,
        responses: Responses<'a>,
    ) -> Result<T, anyhow::Error>
    where
        T: Serialize,
    {
        let parser = PromptParser(prompts.clone());
        Ok(parser
            .template(globals.template(&self.input)?, &responses)?
            .parse()?)
    }
}

impl<T> FromStr for TemplatedInput<T>
where
    T: Default,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            input: s.to_string(),
            marker: Default::default(),
        })
    }
}

impl<'de, T> Visitor<'de> for TemplatedInput<T>
where
    T: Default,
{
    type Value = TemplatedInput<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let need = match std::any::type_name::<T>() {
            "std::str" | "std::str::String" => "string",
            "std::u64" | "std::u16" => "unsigned integer",
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
