mod hls;

use hls::modify_hls_body;

use async_recursion::async_recursion;

use hyper::header::{self};
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request};
use hyper::{Client, Method, Response, StatusCode};
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;

use tokio::net::TcpListener;

use std::collections::HashMap;
use std::time::Duration;
use url::form_urlencoded;

// Todo: implement this
static ALLOWED_HOSTS: [&str; 2] = ["googlevideo.com", "googleusercontent.com"];

// Todo: implement this
static HEADER_BLACKLIST: [&str; 7] = [
    "Accept-Encoding",
    "Authorization",
    "Origin",
    "Referer",
    "Cookie",
    "Set-Cookie",
    "Etag",
];

enum ResponseResult {
    Ok(Response<Body>),
    Err(String),
}
// Sends a HTTP request
#[async_recursion]
async fn send_request(url: &str, host: &str) -> ResponseResult {
    let https = HttpsConnector::new();
    let req_url = &url;

    let client = Client::builder()
        .pool_max_idle_per_host(0)
        .http1_title_case_headers(true)
        .http1_preserve_header_case(true)
        .http2_keep_alive_timeout(Duration::new(60, 0))
        .build::<_, Body>(https);

    let req = Request::builder()
        .uri(req_url.parse::<hyper::Uri>().expect("Error parsing Uri"))
        .method("GET")
        .header("Origin", format!("https://{}", &host))
        .body(Body::empty());

    let request = match req {
        Ok(req) => req,
        Err(e) => return ResponseResult::Err(e.to_string()),
    };

    let res = client.request(request).await;

    let response = match res {
        Ok(res) => res,
        Err(e) => return ResponseResult::Err(e.to_string()),
    };

    // A https://xxxx-xxxx.googlevideo.com/videoplayback?xxxxx
    // URL should return a 206 or 302 Response code.
    // 302 - "Found" should have a "Location" HTTP header.
    if response.headers().contains_key(header::LOCATION) {
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("Error getting Location")
            .to_str()
            .unwrap();

        let redirect = match send_request(&location, &"music.youtube.com").await {
            ResponseResult::Ok(r) => r,
            ResponseResult::Err(e) => return ResponseResult::Err(e.to_string()),
        };
        return ResponseResult::Ok(redirect);
    };

    ResponseResult::Ok(response)
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    response.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        "*".parse::<hyper::http::HeaderValue>().unwrap(),
    );

    let path = &*req.uri().path_and_query().unwrap().path(); // Split the URL Path by "/", and returns each str slice
    let parts: Vec<&str> = path.split("/").collect();

    let query = if let Some(q) = req.uri().path_and_query().unwrap().query() {
        q.to_string()
    } else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("No host parameter provided".into())
            .unwrap());
    };

    // Collect all the URL Search Params into a HashMap
    let query_map = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    // Get the URL Search Param `&host=`
    let host = if let Some(h) = query_map.get("host") {
        h.as_str()
    } else {
        // If `&host=` is not found, return a 400 Bad Request Response
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("No host parameter provided".into())
            .unwrap());
    };

    // Matches Request Method and the first URL Path section
    match (req.method().clone(), parts[1]) {
        (Method::GET, "videoplayback") => {
            let url = format!("https://{}{}?{}", &host, &path, &query).to_string();

            let result = match send_request(&url, &host).await {
                ResponseResult::Ok(r) => r,
                ResponseResult::Err(e) => Response::builder().status(500).body(e.into()).unwrap(),
            };
            *response.body_mut() = result.into_body();
            response.headers_mut().insert(
                hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                "*".parse::<hyper::http::HeaderValue>().unwrap(),
            );

            Ok(response)
        }
        (Method::GET, "api") => {
            let url = format!("https://manifest.googlevideo.com{}", &req.uri().path());

            let res = match send_request(&url, &host).await {
                ResponseResult::Ok(r) => r,
                ResponseResult::Err(e) => response,
            };
            // Collect the inital Response body into bytes,
            // Then turn it into a string
            let body = hyper::body::to_bytes(res.into_body()).await?;
            let body_str = String::from_utf8(body.to_vec()).unwrap();

            // Modify the HLS Manifest body
            let result = modify_hls_body(&body_str, &host)
                .await
                .expect("Could not modify HLS body.");

            // Build a new Response with the modified HLS Manifest
            let result_response = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .body(result.into())
                .unwrap();

            Ok(result_response)
        }
        (Method::GET, "status") => {
            *response.status_mut() = StatusCode::OK;
            Ok(response)
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = "127.0.0.1:33125".parse().unwrap();

    let listener = TcpListener::bind(&addr).await?;

    println!("Server Listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            let service = service_fn(move |req| handle_request(req));

            if let Err(err) = Http::new()
                .http2_keep_alive_timeout(Duration::new(30, 0))
                .http1_preserve_header_case(true)
                .http1_half_close(true)
                .http1_title_case_headers(true)
                .serve_connection(stream, service)
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
