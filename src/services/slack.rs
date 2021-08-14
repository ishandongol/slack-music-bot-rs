use actix_web::{post,web, HttpResponse, Responder,HttpRequest};
use serde::Deserialize;
use regex::Regex;
use lazy_static::lazy_static;
use super::super::{AppState,Song,ws_server};
use actix::Addr;
use mongodb::{bson::doc};

#[derive(Debug,Deserialize)]
struct SlackEvent{
  text:String,
  user:String,
  channel:String,
  r#type: String,
}

#[derive(Debug,Deserialize)]
struct SlackPayload {
  r#type: String,
  challenge: Option<String>,
  event: Option<SlackEvent>,
}

#[post("/events")]
async fn slack_events(_req: HttpRequest,body:web::Json<SlackPayload>,app_state: web::Data<AppState>, srv: web::Data<Addr<ws_server::ChatServer>>) -> impl Responder {
  if body.r#type == "url_verification" {
    let value = body.challenge.as_ref().expect("Not challenged");
    HttpResponse::Ok().body(value)
  } else if body.r#type == "event_callback" {
    let event = body.event.as_ref().expect("No event");
    lazy_static! {
      static ref RE:regex::Regex = Regex::new(r"https?://(www\.)?(youtube)+\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
    }
    if event.r#type == "message" && RE.is_match(&event.text){
      let mut song = Song {
        _id: None,
        url: event.text.trim_start_matches('<').trim_end_matches('>').to_string(),
        user: event.user.to_string(),
        channel: event.channel.to_string(),
        title: None,
        description: None,
      };
      let response = app_state.db.collection("playlist").insert_one(song.clone(),None).await.expect("Failed to create");
      let created_id = response.inserted_id;
      song._id = Some(created_id);
      let stringified_song = serde_json::to_string(&song).unwrap();
      srv.get_ref().clone().do_send(ws_server::NewSong {
        room: "music".to_string(),
        msg: stringified_song,
      });
      println!("{}",&event.text.trim_start_matches('<').trim_end_matches('>'));
    }
    if event.r#type == "app_mention" {
      println!("Mention");
    }
    HttpResponse::Ok().body("OK")
  } else {
    HttpResponse::Ok().body("OK")
  }
}
