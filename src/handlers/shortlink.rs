use std::option::Option;
// use std::collections::HashMap;
// use std::sync::Mutex;

use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::{Json, http::StatusCode};
use base62;
use chrono;
use diesel::prelude::*;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
// use lazy_static::lazy_static;
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
    return base62::encode(number)[..7].to_string();
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
    let now = chrono::Utc::now();
    let mut short_hash = generate_shortlink(&payload.url, Some(&now.to_string())) + "A";

    let mut conn = pool.get().await.map_err(internal_error)?;

    for hash_conflicts in 0..5 {
        // TODO: check with cache first then persist
        let shortlink = schema::shortlink::table
            .filter(schema::shortlink::hash.eq(&short_hash))
            .first::<models::ShortLink>(&mut conn)
            .await
            .optional()
            .map_err(internal_error)?;

        println!("{:?}", shortlink);

        match shortlink {
            Some(_) => {
                let last = short_hash.pop();
                short_hash.push((last.unwrap() as u8 + 1) as char);
            }
            None => break,
        }
        if hash_conflicts == 4 {
            return Err((
                StatusCode::CONFLICT,
                "Shortlink generation failed because of reaching the limit of conflicts, consider providing a custom identifier.".to_string(),
            ));
        }
    }

    let shortlink = diesel::insert_into(schema::shortlink::table)
        .values(models::NewShortlink {
            hash: &short_hash,
            url: &payload.url,
            expire_at: &(now + chrono::TimeDelta::days(10)).naive_utc(),
        })
        .returning(models::ShortLink::as_returning())
        .get_result::<models::ShortLink>(&mut conn)
        .await
        .map_err(internal_error)?;

    // TODO: replace to update cache
    // {
    //     let mut store = MEM_URL_STORE.lock().unwrap();
    //     store
    //         .entry(short_hash.clone())
    //         .or_insert(payload.url.clone());
    // } // Drop the MutexGuard here before await

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

    // TODO: replace to check cache first
    // let store = MEM_URL_STORE.lock().unwrap();
    // match store.get(&short_hash) {
    //     Some(url) => Return Ok(Redirect::to(url)),
    //     None => {},
    // } // Remember drop the MutexGuard before await

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
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
