use actix_web::{get, HttpResponse, Responder,HttpRequest};

#[get("/")]
pub async fn index(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Welcome to slack_music_bot.rs")
}
