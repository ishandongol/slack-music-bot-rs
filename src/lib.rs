use mongodb::Database;
use serde::{Deserialize, Serialize};

mod controllers;

pub use controllers::slack::*;
pub use controllers::index::*;

pub struct AppState {
   pub db: Database,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Song {
  user:String,
  url:String,
  title: Option<String>,
  description:Option<String>,
  channel: String,
}