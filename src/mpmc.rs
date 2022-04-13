use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BroadcastChannel<T: Clone> {
    num_clients: AtomicUsize,
    senders: Mutex<Vec<Sender<T>>>,
    receivers: Mutex<Vec<Receiver<T>>>,
}

impl<T: Clone> Default for BroadcastChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> BroadcastChannel<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            num_clients: AtomicUsize::new(0),
            senders: Mutex::new(vec![]),
            receivers: Mutex::new(vec![]),
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> usize {
        let id = self.num_clients.load(Ordering::SeqCst);
        self.num_clients.store(id + 1, Ordering::SeqCst);

        let (tx, rx) = unbounded();
        self.senders.lock().push(tx);
        self.receivers.lock().push(rx);

        id
    }

    ///# Errors
    /// Can return a vec of all sendErrors. If one is encountered, this will **not** stop the rest
    pub fn send(&self, msg: T) -> Result<(), Vec<SendError<T>>> {
        let mut v = vec![];
        for sender in self.senders.lock().iter() {
            if let Err(e) = sender.send(msg.clone()) {
                v.push(e);
            }
        }

        if v.is_empty() {
            Ok(())
        } else {
            Err(v)
        }
    }

    #[must_use]
    pub fn receive(&self, id: usize) -> Vec<T> {
        let mut v = vec![];
        if let Some(receiver) = self.receivers.lock().get(id) {
            receiver.try_iter().for_each(|t| v.push(t));
        }
        v
    }

    #[must_use]
    pub fn num_subscribed(&self) -> usize {
        self.num_clients.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::mpmc::BroadcastChannel;
    use std::sync::Arc;

    #[test]
    pub fn test_mpmc() {
        let mpmc = Arc::new(BroadcastChannel::new());
        for _ in 0..5 {
            let _ = mpmc.subscribe(); //Can ignore result, because I know precisely how many threads etc.
        }
        for i in 0..5 {
            let mpmc = mpmc.clone();
            std::thread::spawn(move || assert_eq!(mpmc.receive(i), Vec::<i32>::new()));
        }

        mpmc.send(10).unwrap();
        for i in 0..5 {
            let mpmc = mpmc.clone();
            std::thread::spawn(move || assert_eq!(mpmc.receive(i), vec![10]));
        }

        mpmc.send(1).unwrap();
        mpmc.send(2).unwrap();
        for i in 0..5 {
            let mpmc = mpmc.clone();
            std::thread::spawn(move || {
                assert_eq!(mpmc.receive(i), vec![1, 2]);
                assert_eq!(mpmc.receive(i), Vec::<i32>::new());
            });
        }

        assert_eq!(mpmc.receive(100), Vec::<i32>::new());
    }
}
