use {
    crate::types::{tubes::*, Result},
    std::{
        sync::{mpsc, Arc, Mutex},
        thread,
    },
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(return_sender: SendTube<usize>) -> Result<ThreadPool> {
        let size = thread::available_parallelism()?.get();
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&return_sender),
            ));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        return_sender: Arc<Mutex<mpsc::Sender<usize>>>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    return_sender.lock().unwrap().send(id).unwrap();
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

pub fn make_tube<T>() -> Tubes<T> {
    let (r, s) = mpsc::channel();
    (Arc::new(Mutex::new(r)), Arc::new(Mutex::new(s)))
}
