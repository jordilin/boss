extern crate crossbeam_channel;
extern crate num_cpus;

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::ops::Deref;
use std::thread;
use std::thread::JoinHandle;

struct Worker<T> {
    rx: Rx<Work<T>>,
}

enum Work<T> {
    Data(T),
    Quit,
}

struct Rx<T>(Receiver<T>);

impl<T> Deref for Rx<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Receiver<T> {
        &self.0
    }
}

impl<T> Worker<T> {
    fn new(rx: Rx<Work<T>>) -> Self {
        Worker { rx }
    }

    fn run<F>(&self, func: F)
    where
        F: Fn(T),
    {
        loop {
            match self.rx.recv() {
                Some(Work::Data(d)) => func(d),
                Some(Work::Quit) => {
                    break;
                }
                _ => break,
            }
        }
    }
}

pub struct Boss<T> {
    tx: Sender<Work<T>>,
    handles: Vec<JoinHandle<()>>,
    num_workers: usize,
}

impl<T> Boss<T> {
    /// Creates a Boss object in charge of forwarding tasks to internal
    /// workers. The queue_size is optional. If provided the Boss will
    /// block after the queue is full awaiting for workers to finish
    /// their job. You might want to block when there is a lot of data to
    /// be computed by the workers and avoid filling up the machine's
    /// memory. If the queue_size is None, the Boss will send data to the
    /// workers in a non blocking fashion. It will allocate all the data
    /// in memory and await till every worker is done. The number of
    /// worker threads equals the number of CPU cores in the machine.
    pub fn new<F>(queue_size: Option<usize>, func: F) -> Boss<T>
    where
        F: Fn(T) + Send + Copy + 'static,
        T: Send + 'static,
    {
        let num_workers = num_cpus::get();
        let (tx, rx) = match queue_size {
            Some(capacity) => bounded(capacity),
            None => unbounded(),
        };
        let mut handles = vec![];
        for _ in 0..num_workers {
            let rx = rx.clone();
            let handle = thread::spawn(move || {
                Worker::new(Rx(rx)).run(func);
            });
            handles.push(handle);
        }
        Boss {
            tx: tx,
            handles: handles,
            num_workers: num_workers,
        }
    }

    pub fn send_data(&mut self, d: T) {
        self.tx.send(Work::Data(d));
    }

    pub fn finish(self) {
        for _ in 0..self.num_workers {
            self.tx.send(Work::Quit);
        }
        for handle in self.handles {
            handle.join().unwrap();
        }
    }

}
