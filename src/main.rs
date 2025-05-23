
mod config;

use crate::config::Config;

use axum::{
    body::Bytes, 
    extract::Path, 
    http::{
        Extensions, 
        HeaderMap, 
        Method
    }, 
    response::{
        Html, 
        IntoResponse, 
        Response
    }, 
    Router
};

use hyper::{
    StatusCode, 
    Version
};

use reqwest::Client as HTTPClient;

use tokio::net::TcpListener;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {

    let mut port: u16 = 80;

    if let Ok(config) = Config::from_default_location() {
        port = config.router.port;
    }

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", addr);

    let app: Router = Router::new().route("/{*wildcard}", axum::routing::any(handler_wildcard));

    axum::serve(listener, app).await.unwrap();

}

async fn handler_wildcard(headers: HeaderMap, method: Method, Path(path): Path<String>, body: String) -> Response {

    if let Ok(config) = Config::from_default_location() {

        if let Some(internal_url) = config.apply_firewall_rules(&path) {

            let internal_client: HTTPClient = HTTPClient::new();

            match internal_client
                .request(method.clone(), internal_url)
                .headers(headers.clone())
                .body(body)
                .send().await {
                    Ok(internal_response) => {

                        let internal_headers: HeaderMap = internal_response.headers().clone();
                        let internal_extensions: Extensions = internal_response.extensions().clone();
                        let internal_version: Version = internal_response.version().clone();
                        let internal_status: StatusCode = internal_response.status().clone();
                        let internal_body: Bytes = internal_response.bytes().await.unwrap();

                        let pre_response: Response = internal_body.into_response();
                        let (mut parts, body) = pre_response.into_parts();

                        parts.headers = internal_headers;
                        parts.extensions = internal_extensions;
                        parts.version = internal_version;
                        parts.status = internal_status;

                        let post_response: Response = Response::from_parts(parts, body);
                        return post_response;

                    },
                    Err(_) => {
                        return Html("Internal Error!".to_string()).into_response();
                    }
                }

        }
        
    }

    Html(format!("{:?}\n<br>Request-Method: {}\n<br>Request-Path: {}\n<br>Request-Body: {}\n<br>", headers, method, path, body)).into_response()

}
