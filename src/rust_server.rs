use axum::{response::Html, routing::get, Router};
use std::{net::SocketAddr, sync::Arc, thread};
use tokio::net::TcpListener;


fn load_html(path: &str) -> Html<String> {
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|_| "<h1>File not found</h1>".to_string());
    Html(html)
}

pub fn run_server() {
    // --- server config ---use axum::{response::Html, routing::get, Router};
    // use std::{net::SocketAddr, sync::Arc, thread};
    // use tokio::net::TcpListener;
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let html_path = "static/dashboard.html";

    // load once at startup
    let Html(html) = load_html(html_path);
    let html = Arc::new(html);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let html_for_handler = html.clone();

        let app = Router::new().route(
            "/",
            get(move || {
                let html = html_for_handler.clone();
                async move { Html((*html).clone()) }
            }),
        );

        let listener = TcpListener::bind(addr).await.unwrap();
        println!("HTTP server listening on http://{}/", addr);

        axum::serve(listener, app).await.unwrap();
    });
}