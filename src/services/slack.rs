use actix_web::{self,post,web::{self,Payload}, HttpResponse, Responder,HttpRequest};
use serde::{Deserialize,Serialize};
use regex::Regex;
use lazy_static::lazy_static;
use super::super::{AppState,Song,ws_server,YoutubeEmbedResponse};
use actix::Addr;
use mongodb::{bson::doc};
use slack_http_verifier::SlackVerifier;
use std::{env,str};
use futures::StreamExt;

#[derive(Debug,Deserialize,Serialize)]
struct SlackEvent{
  text:String,
  user:String,
  channel:String,
  r#type: String,
}

#[derive(Debug,Deserialize,Serialize)]
struct SlackPayload {
  r#type: String,
  challenge: Option<String>,
  token: Option<String>,
  event: Option<SlackEvent>,
}

const DEFAULT_DATABASE_URL:&str = "--";
const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/events")]
async fn slack_events(req: HttpRequest,mut payload: Payload,app_state: web::Data<AppState>, srv: web::Data<Addr<ws_server::ChatServer>>) -> impl Responder {
  // TODO: create middleware for slack payload verification instead of this
  let slack_secret:String = match env::var("SLACK_SIGNING_SECRET") {
    Ok(val) => val,
    Err(_) => String::from(DEFAULT_DATABASE_URL)
};
  let verifier = SlackVerifier::new(slack_secret).unwrap();
  let ts = req.headers().get("X-Slack-Request-Timestamp").unwrap().to_str().unwrap();
  let sig = req.headers().get("X-Slack-Signature").unwrap().to_str().unwrap();
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
  println!("{:?}",body_string);
  match verifier.verify(&ts,body_string,&sig) {
    Ok(_) => {
      println!("Signature verified");
      if body.r#type == "url_verification" {
        let value = body.challenge.as_ref().expect("Not challenged");
        HttpResponse::Ok().body(value)
      } else if body.r#type == "event_callback" {
        let event = body.event.as_ref().expect("No event");
        lazy_static! {
          static ref RE:regex::Regex = Regex::new(r"https?://(www\.)?(youtube)+\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
        }
        if event.r#type == "message" && RE.is_match(&event.text){
          let url = event.text.trim_start_matches('<').trim_end_matches('>').to_string();
          let resp = reqwest::blocking::get(format!("https://www.youtube.com/oembed?url={}&format=json",url))
            .expect("Filed to get")
            .json::<YoutubeEmbedResponse>()
            .expect("Filed to get");
        println!("{:#?}", resp);
          let mut song = Song {
            _id: None,
            url,
            user: Some(event.user.to_string()),
            channel: Some(event.channel.to_string()),
            title:Some(resp.title),
            thumbnail_url:Some(resp.thumbnail_url),
            description: Some(resp.author_name),
          };
          let response = app_state.db.collection("playlist").insert_one(song.clone(),None).await.expect("Failed to create");
          let created_id = response.inserted_id;
          song._id = Some(created_id);
          song.user= None;
          song.channel=None;
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
    Err(_) => {
      println!("Signature verification failed");
      HttpResponse::Ok().body("Signature Verification failed")
    }
  }
}
