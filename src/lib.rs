//! Unofficial Rust client for [TypeSafe](https://typesafe.ai)'s System One API.
//!
//! Send `state` and typed [`Question`]s (Noul, Choice, Score) to Jev and get
//! back typed [`Answer`]s your code can act on directly, with no text parsing.
//!
//! ```no_run
//! use typesafe_api::{Question, TypeSafeClient};
//!
//! # async fn run() -> Result<(), typesafe_api::Error> {
//! let client = TypeSafeClient::from_env()?;
//!
//! let questions = vec![(
//!     "is_urgent".to_string(),
//!     Question::noul("Does this message convey urgency?"),
//! )];
//!
//! let response = client
//!     .system_one(
//!         "Help! My payouts have been failing for 3 days.",
//!         "jev-latest",
//!         questions,
//!     )
//!     .await?;
//!
//! println!("{:?}", response.get("is_urgent").unwrap().as_noul());
//! # Ok(())
//! # }
//! ```

mod answer;
mod client;
mod error;
mod question;
mod serde_map;

pub use answer::{Answer, ModelCard, ModelsResponse, SystemOneResponse, Usage};
pub use client::{RetryPolicy, TypeSafeClient};
pub use error::Error;
pub use question::{NoulCriteria, Question, Text};
