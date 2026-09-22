//! Mirrors the multi-question example from the TypeSafe quickstart:
//! https://docs.typesafe.ai/introduction/quickstart
//!
//! Run with: TYPESAFE_API_KEY=... cargo run --example ticket_triage

use std::collections::BTreeMap;

use typesafe_api::{Question, TypeSafeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TypeSafeClient::from_env()?;

    let ticket = "Hi, I've been trying to connect my Stripe account for 3 days \
                  and the integration keeps failing. I'm losing sales. Please help ASAP.";

    let mut questions = BTreeMap::new();
    questions.insert(
        "department".to_string(),
        Question::choice(
            "Which team should handle this",
            [
                ("billing", Some("Payment or subscription issues")),
                ("technical", Some("Bugs or integration problems")),
                ("sales", Some("Pricing or account questions")),
            ],
        ),
    );
    questions.insert(
        "frustration".to_string(),
        Question::score(
            "How frustrated the customer appears",
            [
                "Calm, just stating facts",
                "Frustrated but civil",
                "Very angry, strong language",
            ],
        ),
    );
    questions.insert(
        "is_urgent".to_string(),
        Question::noul("The message conveys urgency or time-sensitivity"),
    );

    let response = client.system_one(ticket, "jev-latest", questions).await?;

    let department = response.answers["department"].as_choice().unwrap();
    let frustration = response.answers["frustration"].as_score().unwrap();
    let is_urgent = response.answers["is_urgent"].as_noul().unwrap();

    println!("department:  {department}");
    println!("frustration: {frustration}");
    println!("is_urgent:   {is_urgent}");
    println!("usage:       {:?}", response.usage);

    Ok(())
}
