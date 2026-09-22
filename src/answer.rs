use crate::serde_map;
use serde::Deserialize;

/// A typed answer returned under the same id as its question.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    /// The probability that the answer is yes, from 0 to 1. No separate confidence.
    Noul { noul: f64 },
    /// The selected option, the full distribution, and a confidence score.
    Choice {
        choice: String,
        #[serde(with = "serde_map")]
        probabilities: Vec<(String, f64)>,
        confidence: f64,
    },
    /// A probability-weighted position across the ordered levels; can fall
    /// between two of them.
    Score {
        score: f64,
        #[serde(with = "serde_map")]
        legend: Vec<(String, String)>,
        #[serde(with = "serde_map")]
        probabilities: Vec<(String, f64)>,
        confidence: f64,
    },
}

impl Answer {
    pub fn as_noul(&self) -> Option<f64> {
        match self {
            Answer::Noul { noul } => Some(*noul),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&str> {
        match self {
            Answer::Choice { choice, .. } => Some(choice),
            _ => None,
        }
    }

    pub fn as_score(&self) -> Option<f64> {
        match self {
            Answer::Score { score, .. } => Some(*score),
            _ => None,
        }
    }

    pub fn confidence(&self) -> Option<f64> {
        match self {
            Answer::Choice { confidence, .. } | Answer::Score { confidence, .. } => {
                Some(*confidence)
            }
            Answer::Noul { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// The full response from `POST /v1/systemone`.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemOneResponse {
    /// The versioned model ID that actually answered (e.g. `jev-1.13.0`).
    pub model: String,
    #[serde(with = "serde_map")]
    pub answers: Vec<(String, Answer)>,
    pub usage: Usage,
}

impl SystemOneResponse {
    /// The answer for the given question id, if present.
    pub fn get(&self, id: &str) -> Option<&Answer> {
        self.answers.iter().find(|(k, _)| k == id).map(|(_, v)| v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelCard {
    pub name: String,
    pub description: String,
    pub release_date: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelCard>,
}
