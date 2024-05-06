use axum::{extract::FromRef, response::IntoResponse, routing::get, Router};
use axum_minijinja::{minijinja::{Environment, context}, View, ViewEngine};

#[derive(FromRef, Clone)]
pub struct AppState {
    pub engine: ViewEngine,
}

#[tokio::main]
async fn main() {
    let env = Environment::new();
    let engine = ViewEngine::from_dir(env, "examples/axum-minijinja-simple/views");

    let state = AppState {
        engine,
    };

    let app = Router::new().route("/", get(index)).with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index(view: View) -> impl IntoResponse {
    view.response("index.html", context! {})
}