use axum::{
    Router,
    routing::{get, post},
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/within/a/path", get(|| async { "In a path" }))
        .route("/within/a/path", post(|| async { "Post request" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
