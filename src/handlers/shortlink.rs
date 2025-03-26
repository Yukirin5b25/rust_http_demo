use std::collections::HashMap;
use std::option::Option;
use std::sync::Mutex;

use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::{Json, http::StatusCode};
use base62;
use chrono;
use diesel::prelude::*;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::models;
use crate::schema;

pub fn generate_shortlink(url: &str, identifier: Option<&str>) -> String {
    let target = match identifier {
        Some(identifier) => &(url.to_string() + identifier),
        None => url,
    };
    let hash = Sha256::digest(target.as_bytes());
    let number = u128::from_le_bytes(hash[..16].try_into().unwrap());
    return base62::encode(number)[..8].to_string();
}

// lazy_static! {
//     static ref MEM_URL_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
// }

#[derive(Debug, Serialize, Clone)]
pub struct ShortlinkResponse {
    shortlink: String,
    expire_at: String,
}

#[derive(Deserialize)]
pub struct ShortlinkParams {
    url: String,
}

pub async fn create_shortlink(
    State(pool): State<Pool<AsyncPgConnection>>,
    Json(payload): Json<ShortlinkParams>,
) -> Result<Json<ShortlinkResponse>, (StatusCode, String)> {
    let short_hash = generate_shortlink(&payload.url, None);

    // {
    //     let mut store = MEM_URL_STORE.lock().unwrap();
    //     store
    //         .entry(short_hash.clone())
    //         .or_insert(payload.url.clone());
    // } // Drop the MutexGuard here before await

    let mut conn = pool.get().await.map_err(internal_error)?;

    let shortlink = diesel::insert_into(schema::shortlink::table)
        .values(models::NewShortlink {
            hash: &short_hash,
            url: &payload.url,
            expire_at: &(chrono::Utc::now() + chrono::TimeDelta::days(10)).naive_utc(),
        })
        .returning(models::ShortLink::as_returning())
        .get_result::<models::ShortLink>(&mut conn)
        .await
        .map_err(internal_error)?;

    Ok(Json(ShortlinkResponse {
        shortlink: format!("http://localhost:8080/{}", shortlink.hash),
        expire_at: shortlink.expire_at.to_string(),
    }))
}

pub async fn redirect_shortlink(
    State(pool): State<Pool<AsyncPgConnection>>,
    Path(short_hash): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;

    let shortlink = schema::shortlink::table
        .filter(schema::shortlink::hash.eq(&short_hash))
        .first::<models::ShortLink>(&mut conn)
        .await
        .optional()
        .map_err(internal_error)?;

    match shortlink {
        Some(shortlink) => Ok(Redirect::to(&shortlink.url)),
        None => Err((StatusCode::NOT_FOUND, "Shortlink not found".to_string())),
    }

    // let store = MEM_URL_STORE.lock().unwrap();
    // match store.get(&short_hash) {
    //     Some(url) => Ok(Redirect::to(url)),
    //     None => Err((StatusCode::NOT_FOUND, "Shortlink not found".to_string())),
    // }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
