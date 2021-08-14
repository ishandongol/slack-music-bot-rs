use mongodb::{bson,Database};
use serde::{Deserialize, Serialize};

mod services;
use std::sync::{Arc,atomic::{AtomicUsize}};
pub use services::slack::*;
pub use services::rest::*;
pub use services::websockets::*;

pub struct AppState {
   pub db: Database,
   pub user_count: Arc<AtomicUsize>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Song {
  #[serde(rename="_id",skip_serializing_if="Option::is_none")]
  id: Option<bson::oid::ObjectId>,
  user:String,
  url:String,
  #[serde(skip_serializing_if="Option::is_none")]
  title: Option<String>,
  #[serde(skip_serializing_if="Option::is_none")]
  description:Option<String>,
  channel: String,
}