use actix_web::{post,web, HttpResponse, Responder,HttpRequest};
use serde::Deserialize;
use regex::Regex;
use lazy_static::lazy_static;
use super::super::{AppState,Song};
use mongodb::{bson::doc};

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
async fn slack_events(_req: HttpRequest,body:web::Json<SlackPayload>,app_state: web::Data<AppState>) -> impl Responder {
  if body.r#type == "url_verification" {
    let value = body.challenge.as_ref().expect("No challenged");
    HttpResponse::Ok().body(value)
  } else if body.r#type == "event_callback" {
    let event = body.event.as_ref().expect("No event");
    lazy_static! {
      static ref RE:regex::Regex = Regex::new(r"https?://(www\.)?(youtube)+\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
    }
    if event.r#type == "message" && RE.is_match(&event.text){
      app_state.db.collection("playlist").insert_one(Song {
        url: event.text.to_string(),
        user: event.user.to_string(),
        channel: event.channel.to_string(),
        title: None,
        description: None,
      },None).await.expect("Failed to create");
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
