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
            if inside && ch == DELIMITER {
                inside = false;
                for prompt in &self.collection().to_vec() {
                    if prompt.template == tmp {
                        v.push(prompt.clone())
                    }
                }
                tmp = String::new();
            } else if ch == DELIMITER {
                inside = true
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
            if inside && ch == DELIMITER {
                inside = false;
                for response in &responses {
                    if response.template == tmp {
                        out += &response.to_string()
                    }
                }
                tmp = String::new();
            } else if ch == DELIMITER {
                inside = true
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

#[cfg(test)]
mod tests {
    use super::{Input, InputType, Prompt, PromptCollection, PromptParser};
    use lazy_static::lazy_static;

    lazy_static! {
        static ref PROMPTS: Vec<Prompt> = [
            Prompt {
                template: "greeting".into(),
                question: "how do we greet each other in computers?".into(),
                input_type: InputType::Name,
            },
            Prompt {
                template: "shoesize".into(),
                question: "what is your shoe size?".into(),
                input_type: InputType::Integer,
            },
            Prompt {
                template: "file".into(),
                question: "Give me the name of your favorite file".into(),
                input_type: InputType::Path,
            },
        ]
        .to_vec();
    }

    #[test]
    fn prompt_gathering() {
        let parser = PromptParser(PromptCollection(PROMPTS.clone()));

        assert_eq!(
            *parser
                .prompts("@greeting@".into())
                .unwrap()
                .iter()
                .next()
                .unwrap(),
            PROMPTS[0]
        );

        assert_eq!(
            *parser
                .prompts("also a @greeting@ woo".into())
                .unwrap()
                .iter()
                .next()
                .unwrap(),
            PROMPTS[0]
        );

        // items should appear in order
        assert_eq!(
            *parser
                .prompts("here are three items: @file@ and @shoesize@ and @greeting@ woo".into())
                .unwrap(),
            vec![PROMPTS[2].clone(), PROMPTS[1].clone(), PROMPTS[0].clone()]
        );

        assert_eq!(*parser.prompts("@@".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("@".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("@test".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("@file @shoesize".into()).unwrap(), vec![]);
    }

    #[test]
    fn input_conversion() {
        assert_eq!("20", Input::Integer(20).to_string());
        assert_eq!("-20", Input::SignedInteger(-20).to_string());
        assert_eq!(
            "hello, world!",
            Input::String("hello, world!".into()).to_string()
        );
    }
}
