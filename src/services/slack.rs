use super::super::{fetch_song_info, slack_signature_verification, ws_server, AppState, Song};
use actix::Addr;
use actix_web::{
  self, post,
  web::{self, Payload},
  HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use futures::StreamExt;
use lazy_static::lazy_static;
use mongodb::bson::doc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Debug, Deserialize, Serialize)]
struct SlackEvent {
  text: String,
  user: String,
  channel: String,
  r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackPayload {
  r#type: String,
  challenge: Option<String>,
  token: Option<String>,
  event: Option<SlackEvent>,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/events")]
async fn slack_events(
  req: HttpRequest,
  mut payload: Payload,
  app_state: web::Data<AppState>,
  srv: web::Data<Addr<ws_server::ChatServer>>,
) -> impl Responder {
  let ts = req
    .headers()
    .get("X-Slack-Request-Timestamp")
    .unwrap()
    .to_str()
    .unwrap();
  let sig = req
    .headers()
    .get("X-Slack-Signature")
    .unwrap()
    .to_str()
    .unwrap();
  let mut raw_body = web::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    let chunk = chunk.expect("No chunk");
    // limit max size of in-memory payload
    if (raw_body.len() + chunk.len()) > MAX_SIZE {
      return HttpResponse::Ok().body("MAX SIZE");
    }
    raw_body.extend_from_slice(&chunk);
  }
  let body = serde_json::from_slice::<SlackPayload>(&raw_body).expect("ERror parsign body");
  let raw_body = raw_body.freeze();
  let body_string = str::from_utf8(&raw_body).unwrap();
  println!("{:?}", body_string);

  // TODO: create middleware for slack payload verification instead of this
  let verification_result =
    slack_signature_verification::slack_signature_verification(&ts, body_string, &sig);
  if let Err(err) = verification_result {
    println!("{:?}", err);
    return HttpResponse::Ok().body("signature verification failed");
  }
  println!("Signature verified");
  if body.r#type == "url_verification" {
    let value = body.challenge.as_ref().expect("Not challenged");
    HttpResponse::Ok().body(value)
  } else if body.r#type == "event_callback" {
    let event = body.event.as_ref().expect("No event");
    lazy_static! {
      static ref RE: regex::Regex = Regex::new(
        r"https?://(www\.)?(youtube)+\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"
      )
      .unwrap();
    }
    if event.r#type == "message" && RE.is_match(&event.text) {
      let url = event
        .text
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string();
      let song_info = fetch_song_info::fetch_song_info(&url);
      if let Err(err) = song_info {
        println!("Failed to fetch song info {:?}", err);
        return HttpResponse::Ok().body("Failed to fetch info");
      }
      let song_info = song_info.unwrap();
      let mut song = Song {
        _id: None,
        url,
        user: Some(event.user.to_string()),
        channel: Some(event.channel.to_string()),
        title: Some(song_info.title),
        thumbnail_url: Some(song_info.thumbnail_url),
        description: Some(song_info.author_name),
        shared_on: Utc::now(),
      };
      let response = app_state
        .db
        .collection("playlist")
        .insert_one(song.clone(), None)
        .await
        .expect("Failed to create");
      let created_id = response.inserted_id;
      song._id = Some(created_id);
      song.user = None;
      song.channel = None;
      let stringified_song = serde_json::to_string(&song).unwrap();
      srv.get_ref().clone().do_send(ws_server::NewSong {
        room: "music".to_string(),
        msg: stringified_song,
      });
      println!(
        "{}",
        &event.text.trim_start_matches('<').trim_end_matches('>')
      );
    }
    if event.r#type == "app_mention" {
      println!("Mention");
    }
    HttpResponse::Ok().body("OK")
  } else {
    HttpResponse::Ok().body("OK")
  }
}
