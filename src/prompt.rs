use anyhow::Result;
use serde::{Deserialize, Serialize};

const DELIMITER: char = '@';

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptParser(PromptCollection);

impl PromptParser {
    pub fn collection(&self) -> &PromptCollection {
        &self.0
    }

    pub fn prompts(&self, s: String) -> Result<Vec<Prompt>> {
        let mut v = Vec::new();
        let mut inside = false;
        let mut tmp = String::new();

        for ch in s.chars() {
            if ch == DELIMITER {
                inside = true
            } else if inside && ch == DELIMITER {
                inside = false;
                for prompt in &self.collection().to_vec() {
                    if prompt.template == tmp {
                        v.push(prompt.clone())
                    }
                }
                tmp = String::new();
            } else if inside {
                tmp.push(ch)
            }
        }

        Ok(v)
    }

    pub fn template(&self, s: String, responses: Vec<PromptResponse>) -> Result<String> {
        let mut tmp = String::new();
        let mut inside = false;
        let mut out = String::new();

        for ch in s.chars() {
            if ch == DELIMITER {
                inside = true
            } else if inside && ch == DELIMITER {
                inside = false;
                for response in &responses {
                    if response.template == tmp {
                        out += &response.to_string()
                    }
                }
                tmp = String::new();
            } else if inside {
                tmp.push(ch)
            } else {
                out.push(ch)
            }
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    pub template: String,
    pub question: String,
    pub input_type: InputType,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptCollection(Vec<Prompt>);

impl PromptCollection {
    pub fn to_vec(&self) -> Vec<Prompt> {
        self.0.clone()
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
}

impl ToString for Input {
    fn to_string(&self) -> String {
        match self {
            Input::Integer(x) => x.to_string(),
            Input::SignedInteger(x) => x.to_string(),
            Input::String(x) => x.to_string(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptResponse {
    pub template: String,
    pub input: Input,
}

impl ToString for PromptResponse {
    fn to_string(&self) -> String {
        self.input.to_string()
    }
}
