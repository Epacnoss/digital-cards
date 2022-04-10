use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MpMc<T: Clone> {
    num_clients: AtomicUsize,
    senders: Mutex<Vec<Sender<T>>>,
    receivers: Mutex<Vec<Receiver<T>>>,
}

impl<T: Clone> MpMc<T> {
    pub fn new() -> Self {
        Self {
            num_clients: AtomicUsize::new(0),
            senders: Mutex::new(vec![]),
            receivers: Mutex::new(vec![]),
        }
    }

    pub fn subscribe(&self) -> usize {
        let id = self.num_clients.load(Ordering::SeqCst);
        self.num_clients.store(id + 1, Ordering::SeqCst);

        let (tx, rx) = unbounded();
        self.senders.lock().push(tx);
        self.receivers.lock().push(rx);

        id
    }

    pub fn send(&self, msg: T) {
        for sender in self.senders.lock().iter() {
            sender.send(msg.clone()).unwrap();
        }
    }

    pub fn receive(&self, id: usize) -> Vec<T> {
        let mut v = vec![];
	    if let Some(receiver) = self.receivers.lock().get(id) {
		    receiver.try_iter().for_each(|t| v.push(t));
	    }
        v
    }
}
