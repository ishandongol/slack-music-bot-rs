use actix_web::{get,web, HttpResponse, Responder,HttpRequest};
use super::super::{AppState,Song};
use mongodb::{bson::{doc},options::FindOptions}; 
use futures::stream::{StreamExt};
use chrono::{Utc,DateTime,NaiveDateTime};
use serde::{Deserialize, Serialize};

#[get("/")]
pub async fn index(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Welcome to slack_music_bot.rs")
}

#[derive(Debug, Serialize, Deserialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistQuery {
    #[serde(skip_serializing_if="Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub end_date: Option<String>
}
#[get("/playlist")]
pub async fn playlist(_request: HttpRequest,app_state:web::Data<AppState>,query:web::Query<PlaylistQuery>) -> impl Responder {
    let find_options = FindOptions::builder().projection(doc!{
        "channel":0,
        "user":0,
        "client_message_id":0
    }).build();
    let mut cursor=  app_state.db.collection("playlist").find(doc!{
        "shared_on": {
            "$gte": match &query.start_date {
                Some(start_date) => {
                    let start_date = start_date.parse::<i64>().unwrap();
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(start_date, 0),Utc)
                }
                None => {
                    Utc::today().and_hms(0, 0, 0)
                }
            },
            "$lte":match &query.end_date {
                Some(end_date) => {
                    let end_date = end_date.parse::<i64>().unwrap();
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(end_date, 0),Utc)
                }
                None => {
                    Utc::today().and_hms(23, 59, 59)
                }
            },
        }
    },find_options).await.expect("Failed mongo query");
    let mut playlist:Vec<Song> = Vec::new();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(item) => {
                playlist.push(item)
            }
            Err(e) => println!("{}",e)
        }
    }
    println!("{:?}",playlist);
    HttpResponse::Ok().content_type("application/json").json(playlist).await
}
