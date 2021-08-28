use slack_http_verifier::SlackVerifier;
use std::env;
use std::error::Error;

pub fn verify(
    ts: &str,
    body_string: &str,
    sig: &str,
) -> Result<(), Box<dyn Error>> {
    let slack_secret: String = env::var("SLACK_SIGNING_SECRET")?;
    let verifier = SlackVerifier::new(slack_secret).unwrap(); // panic
    let result = verifier.verify(&ts, body_string, &sig)?;
    Ok(result)
}
