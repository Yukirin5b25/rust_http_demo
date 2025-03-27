use axum::{Json, extract::State, response::IntoResponse};

use crate::state::AppState;

pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let mut config = state.config.clone();

    // hide database password using url::Url
    if let Ok(mut url) = url::Url::parse(&config.database_url) {
        if let Some(_) = url.password() {
            if let Some(password) = url.password() {
                url.set_password(Some(&"*".repeat(password.len()))).ok();
            }
        }
        config.database_url = url.to_string();
    }
    Json(config)
}
