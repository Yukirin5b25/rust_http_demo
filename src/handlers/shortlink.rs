use std::option::Option;

use axum::{Json, response::IntoResponse};
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
    shortlink: String,
}

#[derive(Deserialize)]
pub struct ShortlinkParams {
    url: String,
}

pub async fn create_shortlink(Json(payload): Json<ShortlinkParams>) -> impl IntoResponse {
    Json(Shortlink {
        shortlink: generate_shortlink(&payload.url, None),
    })
}
