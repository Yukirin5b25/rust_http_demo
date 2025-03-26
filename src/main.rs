use std::env;

use axum::{Router, routing::get, routing::post};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use dotenv::dotenv;
use listenfd::ListenFd;
use tokio::net::TcpListener;

use rust_http_demo::handlers::shortlink;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // init resources
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder().build(config).await.unwrap();

    // build app routes
    let app: Router = Router::new()
        .route("/{short_hash}", get(shortlink::redirect_shortlink))
        .route("/shortlink", post(shortlink::create_shortlink))
        .with_state(pool);

    let mut listenfd: ListenFd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind("127.0.0.1:7878").await.unwrap(),
    };

    // run app
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
