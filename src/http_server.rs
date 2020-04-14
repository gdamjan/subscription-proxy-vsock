use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::UnixStream;
use async_std::prelude::*;
use async_std::task;

pub async fn http_listener(addr: &str, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind((addr, port)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("Listening on {}", addr);

    // For each incoming TCP connection, spawn a task to handle it
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let mut stream = stream?;
        let addr = addr.clone();
        task::spawn(async move {
            if let Err(err) = handle_http_request(&addr, &mut stream).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}

async fn handle_http_request(_addr: &str, stream: &mut TcpStream) -> io::Result<()> {
    println!("Got request from {}", stream.peer_addr()?);

    let mut headers = [httparse::EMPTY_HEADER; 16]; // FIXME: more than 16 headers?
    let mut req = httparse::Request::new(&mut headers);

    let bufsize: usize = 64 * 1024; // FIXME: is 64kb enough
    let mut buf = vec![0; bufsize];
    //
    let len = stream.read(&mut buf).await?; // FIXME: how much is read, is it enough?
    eprintln!("{}", len);
    let res = req
        .parse(&buf)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "parsing failed"))?;

    if res.is_partial() && req.path.is_none() {
        return Err(io::Error::new(io::ErrorKind::Other, "parsing partial"));
    };

    // FIXME: routing here: req.path
    let mut vsock = get_subscriber().await?;
    // vsock.write_all(... TODO ...).await?; // send parsed headers
    // vsock.write_all(&buf[body_idx..]).await?;
    vsock.write_all(&buf).await?;


    // IDEALLY use splice(2) here!

    // FIXME: this blocks until the end of the request stream - which is not what we need,
    // and other problems (second request on the same connection etc…)
    // io::copy(stream, &mut vsock).await?; // send the rest of stream to vsock


    // now copy the response to the client
    io::copy(&mut vsock, stream).await?;

    Ok(())
}

async fn get_subscriber() -> io::Result<UnixStream> {
    let (addr, port) = crate::SUBSCRIBERS
        .get()
        .await
        .ok_or(io::Error::new(io::ErrorKind::Other, "no subscribers"))?;

    let mut vsock_stream = UnixStream::connect(addr).await?;

    let connect_cmd = format!("CONNECT {}\n", port);
    vsock_stream.write_all(connect_cmd.as_bytes()).await?;

    // poor mans take_while
    let mut connect_response = Vec::<u8>::new();
    while {
        let mut single_byte = vec![0; 1];
        vsock_stream.read_exact(&mut single_byte).await?;
        connect_response.push(single_byte[0]);
        single_byte != [b'\n']
    } {}

    if !connect_response.starts_with(b"OK ") {
        return Err(io::Error::new(io::ErrorKind::Other, "connect failed"));
    }

    Ok(vsock_stream)
}
