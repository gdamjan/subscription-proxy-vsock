use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::prelude::*;
use async_std::task;
use std::time::Duration;

pub async fn subscription_listener(vm_sock_addr: &str, port: u32) -> std::io::Result<()> {
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

async fn handle_subscription(addr: &str, mut stream: UnixStream) -> std::io::Result<()> {
    // read lines of the form "REGISTER <port>\n"

    let mut register_request = Vec::<u8>::new();
    // poor mans take_while
    while {
        let mut single_byte = vec![0; 1];
        stream.read_exact(&mut single_byte).await?;
        register_request.push(single_byte[0]);
        single_byte != [b'\n']
    } {}
    print!("Subscriber: {}:{}", addr, String::from_utf8_lossy(&register_request));

    /* FIXME
        parse the register command, get the port as u32,
        add (addr, port) to a "subscribers" structure

        let mut iter = buffer.split_ascii_whitespace();
        if let Some("REGISTER") = iter.next() {

        }
        crate::SUBSCRIBERS.lock().await.push_back(…);
    */

    loop {
        stream.write_all(b"ping\n").await?;
        // FIXME: check for pongs, if they fail remove it from subscribers
        task::sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
