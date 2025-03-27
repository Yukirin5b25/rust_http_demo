use crate::config::Config;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool<AsyncPgConnection>,
}

#[derive(Clone)]
pub struct MatricState {
    pub config: Config,
}
