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
            None => return no_subscribers(),
        };

        let mut vsock_stream = UnixStream::connect(addr).await?;
        let connect_cmd = format!("CONNECT {}\n", port);
        print!("{}", connect_cmd);
        vsock_stream.write_all(connect_cmd.as_bytes()).await?;

        // poor mans take_while
        let mut connect_response = Vec::<u8>::new();
        while {
            let mut single_byte = vec![0; 1];
            vsock_stream.read_exact(&mut single_byte).await?;
            connect_response.push(single_byte[0]);
            single_byte != [b'\n']
        } {}

        // FIXME: make sure it's OK
        print!("{}", String::from_utf8_lossy(&connect_response));

        // FIXME: double headers in request and response
        let res = async_h1::client::connect(vsock_stream, req).await?;
        Ok(res)
    })
    .await?;
    Ok(())
}

fn no_subscribers() -> http_types::Result<Response> {
    let mut res = Response::new(StatusCode::BadGateway);
    res.insert_header("Content-Type", "text/plain")?;
    res.set_body("Bad Gateway");
    Ok(res)
}
