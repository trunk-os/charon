use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptParser(String, PromptCollection);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    pub template: String,
    pub question: String,
    pub input_type: InputType,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptCollection(Vec<Prompt>);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    Integer,
    SignedInteger,
    Select(Vec<SelectOption>),
    Name,
    Path,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub name: String,
    pub value: Input,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Input {
    Integer(u64),
    SignedInteger(i64),
    String(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptResponse {
    pub template: String,
    pub input: Input,
}
