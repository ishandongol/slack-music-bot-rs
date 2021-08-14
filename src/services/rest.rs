use actix_web::{get,web, HttpResponse, Responder,HttpRequest};
use super::super::{AppState,Song};
use mongodb::{bson::{doc}}; 
use futures::stream::{StreamExt};

#[get("/")]
pub async fn index(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Welcome to slack_music_bot.rs")
}

#[get("/playlist")]
pub async fn playlist(_request: HttpRequest,app_state:web::Data<AppState>) -> impl Responder {
    let mut cursor=  app_state.db.collection::<Song>("playlist").find(doc!{},None).await.expect("Failed mongo query");
    let mut playlist:Vec<Song> = Vec::new();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(item) => {
                playlist.push(item)
            }
            Err(e) => println!("{}",e)
        }
    }
    HttpResponse::Ok().content_type("application/json").json(playlist)
}