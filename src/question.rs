use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// `instructions` (and some `criteria` entries) accept a plain string, a
/// structured object, or an array. `Text` covers the common string case;
/// `Structured` covers everything else.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Text {
    Plain(String),
    Structured(Value),
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text::Plain(s.to_string())
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::Plain(s)
    }
}

impl From<Value> for Text {
    fn from(v: Value) -> Self {
        Text::Structured(v)
    }
}

/// What a Noul's `true` and `false` outcomes mean. Both fields are optional.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NoulCriteria {
    #[serde(rename = "true", skip_serializing_if = "Option::is_none")]
    pub when_true: Option<Text>,
    #[serde(rename = "false", skip_serializing_if = "Option::is_none")]
    pub when_false: Option<Text>,
}

/// A typed question sent to `POST /v1/systemone`. Every question in a request
/// is evaluated independently against the same `state`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    /// A yes/no question. The answer is the probability the answer is yes.
    Noul {
        instructions: Text,
        #[serde(skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
    /// Picks one option from a fixed set (up to 255). `criteria` maps each
    /// option to a rubric description (`None` when it needs no detail).
    Choice {
        instructions: Text,
        criteria: BTreeMap<String, Option<Text>>,
    },
    /// Rates the state along an ordered rubric (2 to 10 levels).
    Score {
        instructions: Text,
        criteria: Vec<Text>,
    },
}

impl Question {
    pub fn noul(instructions: impl Into<Text>) -> Self {
        Question::Noul {
            instructions: instructions.into(),
            criteria: None,
        }
    }

    pub fn noul_with_criteria(
        instructions: impl Into<Text>,
        when_true: impl Into<Text>,
        when_false: impl Into<Text>,
    ) -> Self {
        Question::Noul {
            instructions: instructions.into(),
            criteria: Some(NoulCriteria {
                when_true: Some(when_true.into()),
                when_false: Some(when_false.into()),
            }),
        }
    }

    pub fn choice<I, K, V>(instructions: impl Into<Text>, options: I) -> Self
    where
        I: IntoIterator<Item = (K, Option<V>)>,
        K: Into<String>,
        V: Into<Text>,
    {
        Question::Choice {
            instructions: instructions.into(),
            criteria: options
                .into_iter()
                .map(|(k, v)| (k.into(), v.map(Into::into)))
                .collect(),
        }
    }

    pub fn score<I, V>(instructions: impl Into<Text>, levels: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Text>,
    {
        Question::Score {
            instructions: instructions.into(),
            criteria: levels.into_iter().map(Into::into).collect(),
        }
    }
}
