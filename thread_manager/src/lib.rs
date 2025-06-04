use std::any::Any;
use std::sync::{mpsc, Arc, Mutex};
use once_cell::sync::Lazy;
use std::thread;

const THREAD_COUNT: usize = 10;

#[derive(Debug)]
struct Channel {
    tx: mpsc::Sender<Box<dyn Any + Send>>,
    rx: Arc<Mutex<mpsc::Receiver<Box<dyn Any + Send>>>>,
}

pub struct ChannelPool {
    channels: [Channel; THREAD_COUNT],
}

impl ChannelPool {
    fn new() -> Self {
        let mut channels = Vec::new();

        for _ in 0..THREAD_COUNT {
            let (tx, rx) = mpsc::channel();
            channels.push(Channel {
                tx,
                rx: Arc::new(Mutex::new(rx)),
            });
        }

        Self {
            channels: channels.try_into().unwrap(),
        }
    }

    pub fn send<T: Any + Send>(&self, thread_id: usize, val: T) {
        match self.channels[thread_id].tx.send(Box::new(val)) {
            Ok(_) => {}
            Err(e) => {
                println!("Send failed: {:?}", e);
            }
        }
    }

    pub fn recv<T: Any>(&self, thread_id: usize) -> Option<T> {
        let rx_lock = self.channels[thread_id].rx.lock().ok()?;
        let msg = rx_lock.recv().ok()?;
        msg.downcast::<T>().ok().map(|b| *b)
    }

    pub fn try_recv<T: Any>(&self, thread_id: usize) -> Option<T> {
        let rx_lock = self.channels[thread_id].rx.lock().ok()?;
        let msg = rx_lock.try_recv().ok()?;
        msg.downcast::<T>().ok().map(|b| *b)
    }
}

static CHANNEL_POOL: Lazy<ChannelPool> = Lazy::new(ChannelPool::new);

pub fn send_msg<T: Any + Send>(thread_id: usize, msg: T) {
    CHANNEL_POOL.send(thread_id, msg);
}

pub fn recv_msg<T: Any>(thread_id: usize) -> Option<T> {
    CHANNEL_POOL.recv(thread_id)
}

pub fn try_recv_msg<T: Any>(thread_id: usize) -> Option<T> {
    CHANNEL_POOL.try_recv(thread_id)
}

pub struct ThreadManager {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl ThreadManager {
    pub fn new(tasks: Vec<Box<dyn FnOnce() + Send>>) -> Self {
        let mut threads = Vec::new();

        for task in tasks {
            let handle = thread::spawn(move || {
                task();
            });
            threads.push(handle);
        }

        ThreadManager { threads }
    }

    pub fn join(self) {
        for t in self.threads {
            t.join().unwrap();
        }
    }
}
