use actix_web::{get,web, HttpResponse, Responder,HttpRequest};
use super::super::{AppState,Song};
use mongodb::{bson::{doc},options::FindOptions}; 
use futures::stream::{StreamExt};

#[get("/")]
pub async fn index(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Welcome to slack_music_bot.rs")
}

#[get("/playlist")]
pub async fn playlist(_request: HttpRequest,app_state:web::Data<AppState>) -> impl Responder {
    let find_options = FindOptions::builder().projection(doc!{
        "channel":0,
        "user":0
    }).build();
    let mut cursor=  app_state.db.collection("playlist").find(doc!{},find_options).await.expect("Failed mongo query");
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
