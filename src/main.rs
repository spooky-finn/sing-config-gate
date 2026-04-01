use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use std::{path::PathBuf, sync::Arc};
use tower_http::trace::TraceLayer;

struct AppState {
    password: String,
    config_dir: PathBuf,
    index_html: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    let port = std::env::var("PORT").unwrap_or("3001".to_string());
    let state = Arc::new(AppState {
        password: std::env::var("PASSWORD").expect("PASSWORD must be set in .env file"),
        config_dir: std::path::Path::new("./config")
            .canonicalize()
            .unwrap_or_else(|_| std::path::Path::new("./config").to_path_buf()),
        index_html: tokio::fs::read_to_string("./index.html")
            .await
            .expect("Failed to load config/index.html"),
    });

    let app = Router::new()
        .route("/{password}/{filename}", get(serve_file))
        .fallback(get(serve_index))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    println!("Starting file server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Html(state.index_html.clone())).into_response()
}

async fn serve_file(
    State(state): State<Arc<AppState>>,
    Path((password, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    if password != state.password {
        return (StatusCode::NOT_FOUND, Html(state.index_html.clone())).into_response();
    }

    let file_path = std::path::Path::new(&state.config_dir).join(&filename);

    let canonical_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, Html(state.index_html.clone())).into_response(),
    };

    if !canonical_path.starts_with(&state.config_dir) {
        return (StatusCode::FORBIDDEN, Html(state.index_html.clone())).into_response();
    }

    match tokio::fs::read_to_string(&canonical_path).await {
        Ok(content) => {
            let content_type = match canonical_path.extension().and_then(|e| e.to_str()) {
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                content,
            )
                .into_response()
        }
        Err(e) => {
            eprintln!("Failed to read file {}: {}", filename, e);
            (StatusCode::NOT_FOUND, Html(state.index_html.clone())).into_response()
        }
    }
}
