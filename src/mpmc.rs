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

#[cfg(test)]
pub mod tests {
    use crate::mpmc::MpMc;
    use std::sync::Arc;
    
    #[test]
    pub fn test_mpmc () {
        let mpmc = Arc::new(MpMc::new());
        for _ in 0..5 {
            mpmc.subscribe(); //Can ignore result, because I know precisely how many threads etc.
        }
        for i in 0..5 {
            let mpmc = mpmc.clone();
            std::thread::spawn(move || {
                assert_eq!(mpmc.receive(i), Vec::<i32>::new())
            });
        }
        
        mpmc.send(10);
        for i in 0..5 {
            let mpmc = mpmc.clone();
            std::thread::spawn(move || {
                assert_eq!(mpmc.receive(i), vec![10])
            });
        }
        
        mpmc.send(1);
        mpmc.send(2);
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