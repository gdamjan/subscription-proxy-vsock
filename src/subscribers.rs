use async_std::sync::{Arc, Mutex};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Subscriber {
    addr: String,
    port: u32,
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        // deregister is async, how do I run it??? task::spawn?
        crate::SUBSCRIBERS.deregister(self.addr.to_string(), self.port);
    }
}

pub struct Subscribers(Arc<Mutex<Vec<Subscriber>>>);

impl Subscribers {
    pub fn new() -> Self {
        let vec = Arc::new(Mutex::new(Vec::new()));
        Self(vec)
    }

    pub async fn get(&self) -> Option<(String, u32)> {
        let vec = self.0.lock().await;
        vec.choose(&mut thread_rng())
            .map(|s| (s.addr.clone(), s.port))
    }

    pub async fn register(&self, addr: String, port: u32) -> Subscriber {
        // let mutex = &self.0;
        // let mut guard = mutex.lock().await;
        // let vec = &mut *guard;
        let vec = &mut *self.0.lock().await;
        vec.push(Subscriber { addr: addr.clone(), port });
        Subscriber { addr, port }
    }

    pub async fn deregister(&self, addr: String, port: u32) {
        let vec = &mut *self.0.lock().await;
        if let Some(i) = vec.iter().position(|v| v.addr == addr && v.port == port) {
            vec.remove(i);
        };
    }
}
