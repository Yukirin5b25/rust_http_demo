# rust_http_demo

The demo implemente a simple shortlink services example in rust, using axum.

## Setup

1. Build the devcontainer.
2. Run `cargo install cargo-watch systemfd` to install `Cargo Watch` and `systmefd`.
3. Run `cargo install diesel_cli` to install `diesel-cli` to manage the shipped pg instance.
4. Run `diesel migration redo` then `diesel migration run` to reset the schema and data in pg.

## Debug

### Normal 

Choose the pre-create launch config `Cargo Launch` to debug the app as a normal rust app.

### Auto-Load

To debug the axum app with auto-reload to free yourself from re-launch the debug, run the Vscode task `Cargo Watch` first. After the axum app is ready, choose and launch with config `Attach to Axum` to attach the debugger to the running app.

The task `Cargo Watch` run this command for you `systemfd --no-pid -s http::8080 -- cargo watch --ignore logs -x run`.

## Ports

By default, `8080` for main application and `7878` for management endpoints like metrics and configs.

## Endpoints

### App

1. Post `localhost:8080/shortlink`, takes a url param to return a shortlink.
2. Get `localhost:8080/{:shorthash}`, will redirect you to the url of the shortlink you visit.

### Management

1. Get `localhost:7878/config`, for list all the configs used by the entire application.
2. Get `localhost:7878/metrics`, for prometheus metircs.
