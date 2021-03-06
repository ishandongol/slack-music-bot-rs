use mongodb::{bson,Database};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
mod services;
mod utils;

use std::sync::{Arc,atomic::{AtomicUsize}};
pub use services::slack::*;
pub use services::rest::*;
pub use services::websockets::*;
pub use utils::*;

pub struct AppState {
   pub db: Database,
   pub user_count: Arc<AtomicUsize>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Song {
  #[serde(skip_serializing_if="Option::is_none")]
  _id: Option<bson::Bson>,
  #[serde(skip_serializing_if="Option::is_none")]
  user:Option<String>,
  url:String,
  #[serde(skip_serializing_if="Option::is_none")]
  title: Option<String>,
  #[serde(skip_serializing_if="Option::is_none")]
  description:Option<String>,
  #[serde(skip_serializing_if="Option::is_none")]
  thumbnail_url:Option<String>,
  #[serde(skip_serializing_if="Option::is_none")]
  channel: Option<String>,
  #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
  shared_on: DateTime<Utc>,
  client_msg_id: String,
}
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct YoutubeEmbedResponse {
  title: String,
  author_name:String,
  thumbnail_url:String,
  author_url: String,
}