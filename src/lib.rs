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

    fn run<F, R>(&self, func: F) -> Vec<R>
    where
        F: Fn(T) -> R,
    {
        let mut results: Vec<R> = Vec::new();
        loop {
            match self.rx.recv() {
                Some(Work::Data(d)) => results.push(func(d)),
                Some(Work::Quit) => {
                    break;
                }
                _ => break,
            }
        }
        results
    }
}

pub struct Boss<T, R> {
    tx: Sender<Work<T>>,
    handles: Vec<JoinHandle<Vec<R>>>,
    num_workers: usize,
}

impl<T, R> Boss<T, R> {
    /// Creates a Boss object in charge of forwarding tasks to internal
    /// workers. The queue_size is optional. If provided the Boss will
    /// block after the queue is full awaiting for workers to finish
    /// their job. You might want to block when there is a lot of data to
    /// be computed by the workers and avoid filling up the machine's
    /// memory. If the queue_size is None, the Boss will send data to the
    /// workers in a non blocking fashion. It will allocate all the data
    /// in memory and await till every worker is done. The number of
    /// worker threads equals the number of CPU cores in the machine.
    pub fn new<F>(queue_size: Option<usize>, func: F) -> Boss<T, R>
    where
        F: Fn(T) -> R + Send + Copy + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let num_workers = num_cpus::get();
        let (tx, rx) = match queue_size {
            Some(capacity) => bounded(capacity),
            None => unbounded(),
        };
        let mut handles = vec![];
        for _ in 0..num_workers {
            let rx = rx.clone();
            let handle = thread::spawn(move || Worker::new(Rx(rx)).run(func));
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

    pub fn finish(self) -> Vec<R> {
        for _ in 0..self.num_workers {
            self.tx.send(Work::Quit);
        }
        let mut results: Vec<R> = vec![];
        for handle in self.handles {
            results.extend(handle.join().unwrap().into_iter());
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fn_test(data: &str) -> Result<String, ()> {
        Ok(format!("Result for {}", data.to_string()))
    }

    #[test]
    fn boss_gather_partial_results() {
        let mut boss = Boss::new(None, fn_test);
        boss.send_data("data 1");
        let results = boss.finish();
        assert_eq!(results[0], Ok("Result for data 1".to_string()))
    }
}
