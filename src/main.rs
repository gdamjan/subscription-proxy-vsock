use async_std::net::{TcpStream, TcpListener};
use async_std::prelude::*;
use async_std::task;
use http_types::{Response, StatusCode};

const HTTP_PORT: u16 = 8888;
const HTTP_ADDR: &str = "127.0.0.1";

#[async_std::main]
async fn main() -> std::io::Result<()> {
    task::spawn(http_listener(HTTP_ADDR, HTTP_PORT)).await?;
    Ok(())
}


async fn http_listener(addr: &str, port: u16) -> std::io::Result<()> {
    // Open up a TCP connection and create a URL.
    let listener = TcpListener::bind((addr, port)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("listening on {}", addr);

    // For each incoming TCP connection, spawn a task to handle it
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let addr = addr.clone();
        task::spawn(async move {
            if let Err(err) = handle_http_request(&addr, stream).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}


// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn handle_http_request(addr: &str, stream: TcpStream) -> http_types::Result<()> {
    println!("starting new connection from {}", stream.peer_addr()?);
    async_h1::accept(addr, stream.clone(), |req| async move {
        req.method(); req.version();
        let mut res = Response::new(StatusCode::Ok);
        res.insert_header("Content-Type", "text/plain")?;
        res.set_body("Hello");
        Ok(res)
    })
    .await?;
    Ok(())
}
