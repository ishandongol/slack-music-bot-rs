use actix_web::{web,App, HttpServer};
use slack_music_bot::{slack_events,index,AppState};
use dotenv::dotenv;

use std::env;
use mongodb::{options::ClientOptions, Client};

const DEFAULT_DATABASE_URL:&str = "mongodb://localhost:21017/music";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url:String = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_DATABASE_URL)
    };
    let mut client_options = ClientOptions::parse(database_url).await.expect("Failed to connect to mongo");
    client_options.app_name = Some("Slack Bot".to_string());
    let client = Client::with_options(client_options).expect("No client");
    let db = client.database("music");
    HttpServer::new(move|| {
        App::new()
            .data(AppState{
                db:db.clone()
            })
            .service(web::scope("/slack")
            .service(slack_events)
        )
            .service(index)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}