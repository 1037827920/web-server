use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    // 增加Option封装，这样可以用take拿走所有权
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// # 函数功能
    /// 创建一个新的线程池
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        // 获得Sender和Receiver
        let (sender, receiver) = mpsc::channel();
        
        // receiver会在多线程中移动，因此要保证线程安全，需要使用Arc和Mutex。Arc可以允许多个Worker同时持有Receiver，而Mutex可以确保一次只有一个Worker能从Receiver中获取任务，防止任务被多次执行
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool { 
            workers, 
            sender: Some(sender)
        }
    }    
    /// # 函数功能
    /// 执行传入的函数f
    pub fn execute<F>(&self, f: F) 
    where
    	F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        // Sender往通道中发送任务
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 主动调用drop关闭sender
        drop(self.sender.take());
        
        for worker in &mut self.workers {
            println!("Shuting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// 闭包的大小编译是未知的，使用Box可以在堆上动态分配内存，从而存储闭包
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    // 因为Worker中的thread字段的JoinHandle类型没有实现copy trait,可以修改Worker的thread字段，使用Option，然后通过take可以拿走内部值的所有权，同时留下一个None
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Receiver会阻塞直到有任务
            let message = receiver.lock().unwrap().recv();     

             match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        // 让每个Worker都拥有自己的唯一id
        Worker { 
            id, 
            thread: Some(thread)
        }
    }
}