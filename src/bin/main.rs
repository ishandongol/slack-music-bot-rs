use actix_web::{web,App, HttpServer};
use slack_music_bot::{slack_events,index,playlist,songs,websocket_route,websocket_user_count,ws_server,AppState};
use dotenv::dotenv;
use actix::*;
use actix_cors::Cors;
use std::{env,sync::{Arc,atomic::{AtomicUsize},}};
use mongodb::{options::ClientOptions, Client};

const DEFAULT_DATABASE_URL:&str = "mongodb://localhost:21017/music";
const DEFAULT_PORT:&str = "5000";
const DEFAULT_ORIGIN:&str = "http://localhost:3000";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url:String = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_DATABASE_URL)
    };
    let port:String = match env::var("PORT") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_PORT)
    };
    let allow_origin:String = match env::var("ALLOWED_ORIGIN") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_ORIGIN)
    };
    let mut client_options = ClientOptions::parse(database_url).await.expect("Failed to connect to mongo");
    client_options.app_name = Some("Slack Bot".to_string());
    let client = Client::with_options(client_options).expect("No client");
    let db = client.database("music");
    let user_count = Arc::new(AtomicUsize::new(0));
    let chat_s = ws_server::ChatServer::new(user_count.clone()).start();
    HttpServer::new(move|| {
        let cors = Cors::default()
        .allowed_origin(&allow_origin)
        .allowed_origin("https://ishandongol.github.io")
        .allowed_origin("https://ishandongol.com.np");
        App::new()
            .data(AppState{
                db:db.clone(),
                user_count: user_count.clone(),
            })
            .data(chat_s.clone())
            .wrap(cors)
            .service(web::scope("/slack")
            .service(slack_events)
            )
            .route("/user-count", web::get().to(websocket_user_count))
            .service(web::resource("/ws")
                .to(websocket_route)    
            )
            .service(index)
            .service(playlist)
            .service(songs)
    })
    .bind(format!("0.0.0.0:{}",port))?
    .run()
    .await
}
