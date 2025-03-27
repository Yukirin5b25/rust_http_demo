use std::future::ready;

use axum::middleware;
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
use rust_http_demo::handlers::{config, metrics::track_metrics, setup_metrics_recorder, shortlink};
use rust_http_demo::state::{AppState, MatricState};

async fn start_main_server() {
    let config = Config::from_env();

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
    let state = AppState { config, pool };
    let app: Router = Router::new()
        .route("/{short_hash}", get(shortlink::redirect_shortlink))
        .route("/shortlink", post(shortlink::create_shortlink))
        .with_state(state)
        .route_layer(middleware::from_fn(track_metrics));

    info!("App listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

async fn start_metrics_server() {
    let config = Config::from_env();

    let metric_state = MatricState { config };
    let recorder_handle = setup_metrics_recorder();

    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    let app: Router = Router::new()
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .route("/config", get(config::get_config))
        .with_state(metric_state);

    info!("Metrics listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

#[tokio::main]
async fn main() {
    // init configs
    dotenv().ok();
    let config = Config::from_env();

    // init logger
    let file_appender = rolling::daily(
        config.logging_file_location.clone(),
        config.logging_file_name.clone(),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = Registry::default()
        .with(EnvFilter::new(config.logging_level.clone()))
        .with(fmt::Layer::new().with_writer(std::io::stdout))
        .with(fmt::Layer::new().with_writer(non_blocking));

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    // run app
    info!("Starting the application...");
    let (_main_server, _metrics_server) = tokio::join!(start_main_server(), start_metrics_server());
}
