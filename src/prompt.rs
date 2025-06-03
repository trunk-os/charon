use crate::{Input, InputType};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const DELIMITER: char = '?';

pub type Responses<'a> = &'a [PromptResponse];

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptParser(pub PromptCollection);

impl PromptParser {
    pub fn collection(&self) -> PromptCollection {
        self.0.clone()
    }

    pub fn prompts(&self, s: String) -> Result<Vec<Prompt>> {
        let mut v = Vec::new();
        let mut inside = false;
        let mut tmp = String::new();

        for ch in s.chars() {
            if inside && ch == DELIMITER {
                inside = false;
                if tmp.is_empty() {
                    // ??, not a template
                    continue;
                }

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

    pub fn template<'a>(&self, s: String, responses: Responses<'a>) -> Result<String> {
        let mut tmp = String::new();
        let mut inside = false;
        let mut out = String::new();

        for ch in s.chars() {
            if inside && ch == DELIMITER {
                inside = false;
                if tmp.is_empty() {
                    // ??, not a template
                    out.push(DELIMITER);
                    continue;
                }

                let mut matched = false;
                for response in responses {
                    if response.template == tmp {
                        out += &response.to_string();
                        matched = true;
                        break;
                    }
                }

                if !matched {
                    return Err(anyhow!("No response matches prompt '{}'", tmp));
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

        // if we were inside at the end of the string, don't swallow the ?
        if inside {
            out += &(DELIMITER.to_string() + &tmp);
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

#[derive(Debug, Clone, Eq, Default, PartialEq, Serialize, Deserialize)]
pub struct PromptCollection(Vec<Prompt>);

impl PromptCollection {
    pub fn to_vec(&self) -> Vec<Prompt> {
        self.0.clone()
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
    use crate::PromptResponse;

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
    fn prompt_responding() {
        let parser = PromptParser(PromptCollection(PROMPTS.clone()));
        assert!(parser.template("?greeting?".into(), &[]).is_err());
        assert!(parser
            .template(
                "?greeting?".into(),
                &[PromptResponse {
                    template: "not-greeting".into(),
                    input: Input::Integer(20)
                }]
            )
            .is_err());
        assert!(parser
            .template(
                "?greeting?".into(),
                &[PromptResponse {
                    template: "greeting".into(),
                    input: Input::String("hello, world!".into())
                }]
            )
            .is_ok());

        assert!(parser
            .template(
                "?greeting?".into(),
                &[
                    PromptResponse {
                        template: "greeting".into(),
                        input: Input::String("hello, world!".into())
                    },
                    PromptResponse {
                        template: "not-greeting".into(),
                        input: Input::String("hello, world!".into())
                    },
                ]
            )
            .is_ok());

        assert_eq!(
            parser
                .template(
                    "?greeting?".into(),
                    &[PromptResponse {
                        template: "greeting".into(),
                        input: Input::String("hello, world!".into())
                    },]
                )
                .unwrap(),
            "hello, world!"
        );

        assert_eq!(
            parser
                .template(
                    "?greeting? ?shoesize?".into(),
                    &[
                        PromptResponse {
                            template: "greeting".into(),
                            input: Input::String("hello, world!".into())
                        },
                        PromptResponse {
                            template: "shoesize".into(),
                            input: Input::Integer(20),
                        }
                    ]
                )
                .unwrap(),
            "hello, world! 20"
        );

        assert!(parser.template("?greeting".into(), &[]).is_ok());
        assert_eq!(
            parser.template("?greeting".into(), &[]).unwrap(),
            "?greeting"
        );
        assert!(parser.template("?".into(), &[]).is_ok());
        assert_eq!(parser.template("?".into(), &[]).unwrap(), "?");
        assert!(parser.template("??".into(), &[]).is_ok());
        assert_eq!(parser.template("??".into(), &[]).unwrap(), "?");
        assert_eq!(
            parser.template("why so serious?".into(), &[]).unwrap(),
            "why so serious?"
        );
        assert_eq!(
            parser.template("why so serious??".into(), &[]).unwrap(),
            "why so serious?"
        );
    }

    #[test]
    fn prompt_gathering() {
        let parser = PromptParser(PromptCollection(PROMPTS.clone()));

        assert_eq!(
            *parser
                .prompts("?greeting?".into())
                .unwrap()
                .iter()
                .next()
                .unwrap(),
            PROMPTS[0]
        );

        assert_eq!(
            *parser
                .prompts("also a ?greeting? woo".into())
                .unwrap()
                .iter()
                .next()
                .unwrap(),
            PROMPTS[0]
        );

        // items should appear in order
        assert_eq!(
            *parser
                .prompts("here are three items: ?file? and ?shoesize? and ?greeting? woo".into())
                .unwrap(),
            vec![PROMPTS[2].clone(), PROMPTS[1].clone(), PROMPTS[0].clone()]
        );

        assert_eq!(*parser.prompts("??".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("?".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("?test".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("?file ?shoesize".into()).unwrap(), vec![]);
        assert_eq!(*parser.prompts("why so serious?".into()).unwrap(), vec![]);
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
