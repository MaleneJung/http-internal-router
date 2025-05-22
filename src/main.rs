
mod firewall;

use crate::firewall::Firewall;

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

use std::{
    env,
    fs, 
    net::SocketAddr
};

#[tokio::main]
async fn main() {

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 80));
    let listener: TcpListener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", addr);

    let app: Router = Router::new().route("/{*wildcard}", axum::routing::any(handler_wildcard));

    axum::serve(listener, app).await.unwrap();

}

async fn handler_wildcard(headers: HeaderMap, method: Method, Path(path): Path<String>, body: String) -> Response {
    
    let mut firewall_raw: String = String::new();
    if let Ok(mut firewall_query) = env::current_exe() {
        firewall_query.pop();
        firewall_query.push("firewall");
        if let Ok(firewall_read) = fs::read_to_string(firewall_query) {
            firewall_raw = firewall_read;
        }
    }

    let firewall: Firewall = Firewall::new(&firewall_raw);

    if let Some(internal_url) = firewall.apply(&path) {

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

    Html(format!("{:?}\n<br>Request-Method: {}\n<br>Request-Path: {}\n<br>Request-Body: {}\n<br>", headers, method, path, body)).into_response()

}
