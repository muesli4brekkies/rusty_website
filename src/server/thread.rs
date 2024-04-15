use std::{
  io,
  sync::{mpsc, Arc, Mutex},
  thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
  _workers: Vec<Worker>,
  sender: mpsc::Sender<Job>,
}

impl ThreadPool {
  // --snip--
  pub fn new(return_sender: Arc<Mutex<mpsc::Sender<usize>>>) -> Result<ThreadPool, io::Error> {
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
      _workers: workers,
      sender,
    })
  }
  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);

    self.sender.send(job).unwrap();
  }
}

struct Worker {
  _id: usize,
  _thread: thread::JoinHandle<()>,
}

impl Worker {
  fn new(
    id: usize,
    receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
    return_sender: Arc<Mutex<mpsc::Sender<usize>>>,
  ) -> Worker {
    let thread = thread::spawn(move || loop {
      let job = receiver.lock().unwrap().recv().unwrap();

      return_sender.lock().unwrap().send(id).unwrap();
      job()
    });

    Worker {
      _id: id,
      _thread: thread,
    }
  }
}
