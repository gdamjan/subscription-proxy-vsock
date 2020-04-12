use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::prelude::*;
use async_std::task;

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
    // read lines of the form "register: <port>\n"
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).await?;

    /*
        parse the register command, get the port as u32,
        add it to list of subscribers
        maybe then loop in ping/pong

        let mut iter = buffer.split_ascii_whitespace();
        if let Some("REGISTER") = iter.next() {

        }
        crate::SUBSCRIBERS.lock().await.push_back(1);
    */
    println!("< {}", buffer);
    Ok(())
}
