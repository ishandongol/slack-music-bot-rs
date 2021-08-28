
use super::super::{ YoutubeEmbedResponse};

pub fn execute (url:&str) -> Result<YoutubeEmbedResponse,Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(format!(
        "https://www.youtube.com/oembed?url={}&format=json",
        url
      ))
     ?
      .json::<YoutubeEmbedResponse>()
      ?;

    Ok(resp)
}