use std::net::TcpListener;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::env;
use num_cpus;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }
    let file_name = args[1].clone();

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("[INFO] Server listening on http://127.0.0.1:8080");
    
    let num_threads = num_cpus::get();
    let pool = ThreadPool::new(num_threads);
    println!("[INFO] Using {} threads", num_threads);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let file_name = file_name.clone();
        pool.execute(move || {
            handle_connection(stream, &file_name);
        });
    }
    println!("[INFO] Server shutting down...");
}

fn handle_connection(mut stream: TcpStream, file_name: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let get = b"GET / HTTP/1.1\r\n";
    if buffer.starts_with(get) {
        let contents = fs::read_to_string(file_name).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );
        stream.write(response.as_bytes()).unwrap();
    } else {
        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }
    stream.flush().unwrap();
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        println!("[INFO] Creating {} worker threads", size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("[INFO] Shutting down thread pool...");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        println!("[INFO] Thread pool shutdown complete.");
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<std::sync::mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            println!("[INFO] Worker {}: Started", id);
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("[DEBUG] Worker {}: Processing new request", id);
                        job();
                        println!("[DEBUG] Worker {}: Request processed", id);
                    }
                    Message::Terminate => {
                        println!("[INFO] Worker {}: Shutting down", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

