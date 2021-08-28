use actix_web::web::{BytesMut, Payload};
use futures::StreamExt;
use std::fmt;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Debug, Clone)]
pub struct PayloadError;

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Payload Max Size")
    }
}

pub async fn parse(mut payload: Payload) -> Result<BytesMut, PayloadError> {
    let mut raw_body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.expect("No chunk");
        // limit max size of in-memory payload
        if (raw_body.len() + chunk.len()) > MAX_SIZE {
            return Err(PayloadError);
        };
        raw_body.extend_from_slice(&chunk);
    }
    Ok(raw_body)
}
