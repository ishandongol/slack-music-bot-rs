use mongodb::Database;
use serde::{Deserialize, Serialize};

mod services;
use std::sync::{Arc,atomic::{AtomicUsize}};
pub use services::slack::*;
pub use services::index::*;
pub use services::websockets::*;

pub struct AppState {
   pub db: Database,
   pub user_count: Arc<AtomicUsize>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Song {
  user:String,
  url:String,
  title: Option<String>,
  description:Option<String>,
  channel: String,
}