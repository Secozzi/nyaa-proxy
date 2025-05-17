mod error;

use std::collections::HashSet;
use std::{env, str};
use std::net::SocketAddr;
use axum::{
  routing::get,
  extract::{Request, State},
  Router,
};
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use reqwest::Client;
use reqwest::header::HeaderName;
use crate::error::Error;

#[derive(Clone)]
struct AppState {
    client: Client,
    nyaa_url: String,
    proxy_url: String,
    excluded_headers: HashSet<HeaderName>,
}

const NYAA_URL: &str = "https://nyaa.si";

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    
    let nyaa_url = env::var("NYAA_URL")
        .unwrap_or(NYAA_URL.to_owned());
    
    let proxy_url = env::var("PROXY_URL")
        .unwrap_or(NYAA_URL.to_owned());
    
    let client = Client::new();
    let excluded_headers = excluded_headers();
    let state = AppState { client, nyaa_url, proxy_url, excluded_headers };

    let app = Router::new()
        .route("/", get(handler))
        .route("/{*path}", get(handler))
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await.unwrap();
}

async fn handler(State(state): State<AppState>, req: Request) -> Response {
    let url = format!("{}{}", state.nyaa_url, req.uri());
    
    let resp = match state.client.get(url).send().await {
        Ok(resp) => resp,
        Err(e) => return Error::BadGateway(e).into_response(),
    };

    let mut response_builder = Response::builder().status(resp.status());
    let mut is_html = false;
    
    for (name, value) in resp.headers() {
        if !state.excluded_headers.contains(name) {
            response_builder = response_builder.header(name, value);
        }
        
        if name.as_str().eq_ignore_ascii_case("content-type") {
            if let Ok(value_str) = value.to_str() {
                is_html = value_str.to_lowercase().starts_with("text/html");
            }
        }
    }

    let body_bytes = match resp.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return Error::ReadFailure.into_response(),
    };

    let body = if is_html {
        match process_html_body(&body_bytes, &state.nyaa_url, &state.proxy_url) {
            Ok(body) => body,
            Err(error_response) => return error_response,
        }
    } else {
        Body::from(body_bytes)
    };

    response_builder
        .body(body)
        .unwrap_or_else(|_| Error::CreateResponseFailure.into_response())
}

fn process_html_body(body_bytes: &[u8], nyaa_url: &str, proxy_url: &str) -> Result<Body, Response<Body>> {
    let body_str = match str::from_utf8(body_bytes) {
        Ok(s) => s,
        Err(_) => return Err(Error::BadUtf8.into_response()),
    };

    let injected = format!(r#"<div id="snackbar">Copied link</div>
    <style>#snackbar{{font-family:"Helvetica Neue",Helvetica,Arial,sans-serif;visibility:hidden;min-width:100px;margin-left:-50px;background-color:#333;color:#fff;text-align:center;border-radius:8px;padding:16px;position:fixed;z-index:1;left:50%;bottom:30px;font-size:17px}}#snackbar.show{{visibility:visible;-webkit-animation:.5s fadein,.5s 2.5s fadeout;animation:.5s fadein,.5s 2.5s fadeout}}@-webkit-keyframes fadein{{from{{bottom:0;opacity:0}}to{{bottom:30px;opacity:1}}}}@keyframes fadein{{from{{bottom:0;opacity:0}}to{{bottom:30px;opacity:1}}}}@-webkit-keyframes fadeout{{from{{bottom:30px;opacity:1}}to{{bottom:0;opacity:0}}}}@keyframes fadeout{{from{{bottom:30px;opacity:1}}to{{bottom:0;opacity:0}}}}</style>
    <script>const rssLink=document.querySelector('a[href*="/?page=rss"]');rssLink.addEventListener("click",e=>{{e.preventDefault(),navigator.clipboard.writeText(e.currentTarget.href.replace("{proxy_url}","{nyaa_url}"));var a=document.getElementById("snackbar");a.className="show",setTimeout(function(){{a.className=a.className.replace("show","")}},3e3)}});</script></body>"#);
    
    let modified_body = body_str
        .replace(nyaa_url, proxy_url)
        .replacen("</body>", &injected, 1);
    
    Ok(Body::from(modified_body))
}

fn excluded_headers() -> HashSet<HeaderName> {
    let headers = [
        "transfer-encoding",
        "content-type",
        "content-length",
        "content-encoding",
        "cache-control",
    ];

    headers.iter()
        .filter_map(|h| HeaderName::from_bytes(h.as_bytes()).ok())
        .collect()
}
