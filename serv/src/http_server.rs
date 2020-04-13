use std::path::PathBuf;
use std::net::SocketAddr;
use std::future::Future;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use hyper::{
    server::conn::AddrStream,
    Body, Method, Response, StatusCode, Request,
};

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOT_FOUND: &[u8] = b"Not Found";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub clnt_deploy_dir: PathBuf,
}

#[derive(Clone)]
pub struct Server {
    config: Arc<Config>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub fn serve(&self) -> impl Future<Output=Result<(), hyper::Error>> + '_ {
        let make_service = hyper::service::make_service_fn(move |_: &AddrStream| {
            let config = self.config.clone();

            async move {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                    service(config.clone(), req)
                }))
            }
        });

        hyper::Server::bind(&self.config.listen_addr)
            .serve(make_service)
    }
}

async fn service(config: Arc<Config>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve static files
        (&Method::GET, "/") | (&Method::GET, "/index.html") => 
            send_file(config, "index.html", "text/html").await,
        (&Method::GET, "/clnt.js") =>
            send_file(config, "clnt.js", "text/javascript").await,
        (&Method::GET, "/clnt.wasm") =>
            send_file(config, "clnt.wasm", "application/wasm").await,

        // Return 404 Not Found for other routes
        _ => Ok(not_found()),
    }
}

/// Serve a file.
/// 
/// TODO: We'll need to cache the files eventually, but for now reloading
/// allows for quicker development.
/// 
/// Source: https://github.com/hyperium/hyper/blob/master/examples/send_file.rs
async fn send_file(config: Arc<Config>, filename: &str, content_type: &str) -> Result<Response<Body>, hyper::Error> {
    // Serve a file by asynchronously reading it entirely into memory.
    // Uses tokio_fs to open file asynchronously, then tokio::io::AsyncReadExt
    // to read into memory asynchronously.

    let filename = config.clnt_deploy_dir.join(filename);

    if let Ok(mut file) = File::open(filename).await {
        let mut buf = Vec::new();

        if let Ok(_) = file.read_to_end(&mut buf).await {
            let response = Response::builder()
                .header("Content-Type", content_type)
                .body(buf.into())
                .unwrap();
            Ok(response)
        } else {
            Ok(internal_server_error())
        }
    } else {
        Ok(not_found())
    }
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOT_FOUND.into())
        .unwrap()
}

fn internal_server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(INTERNAL_SERVER_ERROR.into())
        .unwrap()
}