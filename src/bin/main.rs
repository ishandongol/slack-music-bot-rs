use actix_web::{App, HttpServer};
use slack_music_bot::{slack_events,index};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(slack_events)
            .service(index)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}