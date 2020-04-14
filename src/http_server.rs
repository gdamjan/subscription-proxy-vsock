use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::UnixStream;
use async_std::prelude::*;
use async_std::task;
use http_types::{Response, StatusCode};

pub async fn http_listener(addr: &str, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind((addr, port)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("Listening on {}", addr);

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

async fn handle_http_request(addr: &str, stream: TcpStream) -> http_types::Result<()> {
    println!("Got request from {}", stream.peer_addr()?);
    async_h1::accept(addr, stream.clone(), |req| async move {
        let (addr, port) = match crate::SUBSCRIBERS.get().await {
            Some(t) => t,
            None => return bad_gateway(),
        };

        let mut vsock_stream = match UnixStream::connect(addr).await {
            Ok(v) => v,
            Err(_) => return bad_gateway(),
        };

        let connect_cmd = format!("CONNECT {}\n", port);
        if vsock_stream
            .write_all(connect_cmd.as_bytes())
            .await
            .is_err()
        {
            return bad_gateway();
        }

        // poor mans take_while
        let mut connect_response = Vec::<u8>::new();
        while {
            let mut single_byte = vec![0; 1];
            if vsock_stream.read_exact(&mut single_byte).await.is_err() {
                return bad_gateway();
            }
            connect_response.push(single_byte[0]);
            single_byte != [b'\n']
        } {}

        if !connect_response.starts_with(b"OK ") {
            return bad_gateway();
        }

        // send request to subscriber! FIXME: does it stream the request body?
        let mut req = async_h1::client::Encoder::encode(req).await?;
        io::copy(&mut req, &vsock_stream).await?;

        // FIXME: how to avoid this? just return the stream as-is
        // not to say that decode adds double headers :/
        let res = async_h1::client::decode(vsock_stream).await?;
        Ok(res)

        // Ideally this would be:
        // Ok(vsock_stream)
    })
    .await?;
    Ok(())
}

fn bad_gateway() -> http_types::Result<Response> {
    // Err(http_types::Error::from_str(StatusCode::BadGateway, "Bad Gateway")
    let mut res = Response::new(StatusCode::BadGateway);
    res.insert_header("Content-Type", "text/plain")?;
    res.set_body("Bad Gateway");
    Ok(res)
}
