use std::fmt;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker{
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

enum Message {
    NewJob(Job),
    Terminate
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");
        for worker in &mut self.workers{
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread =  thread::spawn(move || {
            loop {
                let message = reciever.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                    }
                    Message::Terminate =>{
                        println!("Worker {} was told to terminate.",id);
                        break;
                    }
                }
            }
        });
        Worker{id,thread: Some(thread)}
    }
}

pub struct PoolCreationError{
    msg: String
}

impl fmt::Display for PoolCreationError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "{}", self.msg.as_str())
    }
}

impl fmt::Debug for PoolCreationError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError>{
        if size <=0 {
            let error_msg = String::from("Passed in invalid size, must be greater than 0");
            return Err(PoolCreationError{msg: error_msg});
        }

        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id,Arc::clone(&reciever)));
        }
        Ok(ThreadPool{workers, sender})
    }

    pub fn execute<F>(&self, f:F) where F: FnOnce() + Send + 'static{
        let job =  Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}
