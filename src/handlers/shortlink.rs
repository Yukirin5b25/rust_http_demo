use std::option::Option;

use axum::response::{IntoResponse, Redirect};
use axum::{Json, extract::Path};
use base62;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub fn generate_shortlink(url: &str, identifier: Option<&str>) -> String {
    let target = match identifier {
        Some(identifier) => &(url.to_string() + identifier),
        None => url,
    };
    let hash = Sha256::digest(target.as_bytes());
    let number = u128::from_le_bytes(hash[..16].try_into().unwrap());
    return base62::encode(number)[..8].to_string();
}

#[derive(Debug, Serialize, Clone)]
struct Shortlink {
    short_hash: String,
    short_link: String,
}

#[derive(Deserialize)]
pub struct ShortlinkParams {
    url: String,
}

pub async fn create_shortlink(Json(payload): Json<ShortlinkParams>) -> impl IntoResponse {
    let short_hash = generate_shortlink(&payload.url, None);
    let short_link = format!("http://localhost:8080/{}", short_hash);
    Json(Shortlink {
        short_hash: short_hash,
        short_link: short_link,
    })
}

pub async fn redirect_to_google(Path(short_hash): Path<String>) -> impl IntoResponse {
    println!(
        "TODO: Should redirect to http://localhost:8080/{} instead",
        short_hash
    );
    Redirect::to("https://www.google.com")
}
