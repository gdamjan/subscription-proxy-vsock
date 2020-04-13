use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::prelude::*;
use async_std::{future, task};
use std::io;
use std::time::Duration;

pub async fn subscription_listener(vm_sock_addr: &str, port: u32) -> io::Result<()> {
    let listen_addr = format!("{}_{}", vm_sock_addr, port);
    let listener = UnixListener::bind(&listen_addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let addr = vm_sock_addr.to_owned();
        task::spawn(async move {
            if let Err(err) = handle_subscription(&addr, stream).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}

async fn handle_subscription(addr: &str, mut stream: UnixStream) -> io::Result<()> {
    // read lines of the form "REGISTER <port>\n"
    const REGISTER: &[u8] = b"REGISTER ";

    let mut register_request = Vec::<u8>::new();
    // poor mans take_while
    while {
        let mut single_byte = vec![0; 1];
        stream.read_exact(&mut single_byte).await?;
        register_request.push(single_byte[0]);
        single_byte != [b'\n']
    } {}

    let port: u32 = if register_request.starts_with(REGISTER) && register_request.ends_with(b"\n") {
        let start = REGISTER.len();
        let end = register_request.len() - 1;
        let s = String::from_utf8_lossy(&register_request[start..end]);
        s.parse::<u32>()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Bad REGISTER request"))?
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Bad REGISTER request"));
    };
    println!("Registered: {}:{}", addr, port);

    /* FIXME
        add (addr, port) to a "subscribers" structure
        crate::SUBSCRIBERS.lock().await.push_back(…);
    */

    loop {
        let mut response: Vec<u8> = vec![0; 5];
        stream.write_all(b"ping\n").await?;
        if future::timeout(Duration::from_secs(5), stream.read_exact(&mut response))
            .await
            .is_err()
        {
            break;
        }
        if response != b"pong\n" {
            break;
        }
        task::sleep(Duration::from_secs(5)).await;
    }
    // FIXME: deregister
    println!("De-registered: {}:{}", addr, port);

    Ok(())
}
