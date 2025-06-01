use serde::{de::Visitor, Deserialize, Serialize};

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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TemplatedInput {
    pub input: String,
    pub input_type: InputType,
}

impl<'de> Visitor<'de> for TemplatedInput {
    type Value = Input;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let need = match &self.input_type {
            InputType::Path | InputType::Name => "string",
            InputType::Integer => "unsigned integer",
            InputType::SignedInteger => "signed integer",
            InputType::Boolean => "boolean",
            InputType::Select(options) => &format!(
                "one of the following options: [{}]",
                options
                    .iter()
                    .map(|e| e.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        };

        formatter.write_str(&format!("expecting a string that parses as {}", need))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match &self.input_type {
            InputType::Path | InputType::Name => Input::String(v.to_string()),
            InputType::Integer => Input::Integer(v.parse().map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
            })?),
            InputType::Boolean => Input::Boolean(v.parse().map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
            })?),
            InputType::Select(options) => {
                for o in options {
                    if &o.name == v {
                        return Ok(o.value.clone());
                    }
                }
                Input::Null
            }
            InputType::SignedInteger => Input::SignedInteger(v.parse().map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
            })?),
        })
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Input::String(s) => Ok(serializer.serialize_str(s)?),
            Input::Integer(i) => Ok(serializer.serialize_u64(*i)?),
            Input::SignedInteger(i) => Ok(serializer.serialize_i64(*i)?),
            Input::Boolean(b) => Ok(serializer.serialize_bool(*b)?),
            Input::Null => Ok(serializer.serialize_none()?),
        }
    }
}
