# typesafe-api

Unofficial Rust client for [TypeSafe](https://typesafe.ai)'s System One API.

Send `state` (any `Serialize`-able value) and typed [`Question`]s — Noul, Choice, Score — to
a Jev model and get back typed [`Answer`]s your code can act on directly, with no text
parsing.

## Install

```bash
cargo add typesafe-api
```

## Usage

Set `TYPESAFE_API_KEY` in your environment, then:

```rust,no_run
use typesafe_api::{Question, TypeSafeClient};

# async fn run() -> Result<(), typesafe_api::Error> {
let client = TypeSafeClient::from_env()?;

let questions = vec![(
    "is_urgent".to_string(),
    Question::noul("Does this message convey urgency?"),
)];

let response = client
    .system_one(
        "Help! My payouts have been failing for 3 days.",
        "jev-latest",
        questions,
    )
    .await?;

println!("{:?}", response.get("is_urgent").unwrap().as_noul());
# Ok(())
# }
```

Every question in a `system_one` call is evaluated independently and in parallel against
the same `state`, so batch everything you need into one call rather than issuing several.

See [`examples/ticket_triage.rs`](examples/ticket_triage.rs) for a multi-question example
(department routing, a frustration score, and an urgency check run together on one
support ticket):

```bash
TYPESAFE_API_KEY=... cargo run --example ticket_triage
```

## Question types

- `Question::noul(instructions)` — a yes/no question; the answer is the probability the
  answer is yes. `Question::noul_with_criteria` lets you spell out what "true" and "false"
  mean.
- `Question::choice(instructions, options)` — picks one option from a fixed set (up to
  255), each optionally paired with a rubric description.
- `Question::score(instructions, levels)` — rates the state along an ordered rubric (2 to
  10 levels).

## Configuration

`TypeSafeClient::new(api_key)` builds a client with an explicit key;
`TypeSafeClient::from_env()` reads it from `TYPESAFE_API_KEY`. `with_base_url` overrides the
API base URL (useful for testing against a mock server), and `with_retry_policy` controls
retry behavior for `429`/`529` responses (retried with exponential backoff by default,
mirroring TypeSafe's own SDKs; use `RetryPolicy::none()` to disable).

## License

MIT
