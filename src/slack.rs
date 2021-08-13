use actix_web::{post,web, HttpResponse, Responder,HttpRequest};
use serde::Deserialize;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Deserialize)]
#[derive(Debug)]
struct SlackEvent{
  text:String,
  user:String,
  channel:String,
  r#type: String,
}

#[derive(Deserialize)]
#[derive(Debug)]
struct SlackPayload {
  r#type: String,
  challenge: Option<String>,
  event: Option<SlackEvent>,
}

#[post("/slack/events")]
async fn slack_events(_req: HttpRequest,body:web::Json<SlackPayload>) -> impl Responder {
  if body.r#type == "url_verification" {
    let value = body.challenge.as_ref().expect("No challenged");
    HttpResponse::Ok().body(value)
  } else if body.r#type == "event_callback" {
    let event = body.event.as_ref().expect("No event");
    lazy_static! {
      static ref RE:regex::Regex = Regex::new(r"https?://(www\.)?(youtube)+\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
    }
    if event.r#type == "message" && RE.is_match(&event.text){
      println!("{}",&event.text);
    }
    if event.r#type == "app_mention" {
      println!("Mention");
    }
    HttpResponse::Ok().body("OK")
  } else {
    HttpResponse::Ok().body("OK")
  }
}
