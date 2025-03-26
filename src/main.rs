use axum::{Router, response::Html, routing::get, routing::post};
use listenfd::ListenFd;
use tokio::net::TcpListener;

use rust_http_demo::handlers::shortlink;

#[tokio::main]
async fn main() {
    // build app routes
    let app: Router = Router::new()
        .route("/{short_link}", get(shortlink::redirect_to_google))
        .route("/shortlink", post(shortlink::create_shortlink));

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

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
