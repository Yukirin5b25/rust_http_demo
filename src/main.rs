use axum::{Router, routing::get, routing::post};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use dotenv::dotenv;

use tokio::net::TcpListener;

use rust_http_demo::config::Config;
use rust_http_demo::handlers::shortlink;
use rust_http_demo::state::AppState;

#[tokio::main]
async fn main() {
    // init configs
    dotenv().ok();
    let config = Config::from_env();

    // init resources
    let pool_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        config.database_url.clone(),
    );
    let pool = Pool::builder().build(pool_config).await.unwrap();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.axum_server_port))
        .await
        .unwrap();

    // build app routes
    let state = AppState { config, pool };
    let app: Router = Router::new()
        .route("/{short_hash}", get(shortlink::redirect_shortlink))
        .route("/shortlink", post(shortlink::create_shortlink))
        .with_state(state);

    // run app
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
