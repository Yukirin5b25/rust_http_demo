use axum::{Router, routing::get, routing::post};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use dotenv::dotenv;

use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

use listenfd::ListenFd;
use tokio::net::TcpListener;

use rust_http_demo::config::Config;
use rust_http_demo::handlers::{config, shortlink};
use rust_http_demo::state::AppState;

#[tokio::main]
async fn main() {
    // init configs
    dotenv().ok();
    let config = Config::from_env();

    // init logger
    let file_appender = rolling::daily("logs", "app.log"); // Logs will be written to logs/app.log
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = Registry::default()
        .with(EnvFilter::new("info")) // Set log level to Info
        .with(fmt::Layer::new().with_writer(std::io::stdout)) // Log to stdout
        .with(fmt::Layer::new().with_writer(non_blocking)); // Log to file

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    // init resources
    let pool_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        config.database_url.clone(),
    );
    let pool = Pool::builder().build(pool_config).await.unwrap();
    let mut listenfd: ListenFd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind(format!("127.0.0.1:{}", config.axum_server_port))
            .await
            .unwrap(),
    };

    // build app routes
    let state = AppState { config, pool };
    let app: Router = Router::new()
        .route("/{short_hash}", get(shortlink::redirect_shortlink))
        .route("/shortlink", post(shortlink::create_shortlink))
        .route("/config", get(config::get_config))
        .with_state(state);

    // run app
    info!("Starting the application...");
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
