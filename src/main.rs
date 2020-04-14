#[macro_use]
extern crate lazy_static;

use async_std::task;

const HTTP_PORT: u16 = 8888;
const HTTP_ADDR: &str = "127.0.0.1";
const VSOCK_PORT: u32 = 12345;
const VSOCK_ADDR: &str = "/tmp/cid10.sock";

mod http_server;
mod subscribers;
mod subscription_server;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let h = task::spawn(http_server::http_listener(HTTP_ADDR, HTTP_PORT));
    let v = task::spawn(subscription_server::subscription_listener(
        VSOCK_ADDR, VSOCK_PORT,
    ));

    h.await?;
    v.await?;
    Ok(())
}

// Global singleton
lazy_static! {
    pub static ref SUBSCRIBERS: subscribers::Subscribers = subscribers::Subscribers::new();
}
