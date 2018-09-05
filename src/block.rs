use crossbeam_channel::{bounded, unbounded, Sender};
use num_cpus;
use std::thread;
use std::thread::JoinHandle;

use data::{Rx, Work};

struct Worker<T> {
    rx: Rx<Work<T>>,
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

/// CSPWorkerPool stands for communicating sequencial processes worker pool
/// using the crossbeam_channel crate. It creates a simple abstraction where
/// clients just instantiate the pool, send data to it and receive it in an
/// easy to use and transparent way.
pub struct CSPWorkerPool<T, R> {
    tx: Sender<Work<T>>,
    handles: Vec<JoinHandle<Vec<R>>>,
    num_workers: usize,
}

impl<T, R> CSPWorkerPool<T, R> {
    /// Creates a CSPWorkerPool object in charge of forwarding tasks to
    /// internal workers. The number of threads (workers) is optional. If
    /// not provided, it will default to the number of cores in the
    /// machine. The queue_size is optional too. If provided the
    /// CSPWorkerPool will block after the queue is full awaiting for
    /// workers to finish their job. You might want to block when there is
    /// a lot of data to be computed by the workers and avoid filling up
    /// the machine's memory. If the queue_size is None, the CSPWorkerPool
    /// will send data to the workers in a non blocking fashion. It will
    /// allocate all the data in memory and await till every worker is
    /// done. The number of worker threads equals the number of CPU cores
    /// in the machine.
    pub fn new<F>(
        num_threads: Option<usize>,
        queue_size: Option<usize>,
        func: F,
    ) -> CSPWorkerPool<T, R>
    where
        F: Fn(T) -> R + Send + Copy + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = match queue_size {
            Some(capacity) => bounded(capacity),
            None => unbounded(),
        };
        let mut handles = vec![];
        let num_workers = match num_threads {
            Some(val) => val,
            None => num_cpus::get(),
        };
        for _ in 0..num_workers {
            let rx = rx.clone();
            let handle = thread::spawn(move || Worker::new(Rx(rx)).run(func));
            handles.push(handle);
        }
        CSPWorkerPool {
            tx: tx,
            handles: handles,
            num_workers: num_workers,
        }
    }

    pub fn send_data(&self, d: T) {
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
    fn boss_gather_partial_results_default_threads_unbounded() {
        let boss = CSPWorkerPool::new(None, None, fn_test);
        boss.send_data("data 1");
        let results = boss.finish();
        assert_eq!(results[0], Ok("Result for data 1".to_string()))
    }

    #[test]
    fn boss_gather_partial_results_with_threads() {
        let boss = CSPWorkerPool::new(Some(2), None, fn_test);
        boss.send_data("data 1");
        let results = boss.finish();
        assert_eq!(results[0], Ok("Result for data 1".to_string()))
    }

    #[test]
    fn boss_gather_partial_results_with_bounded_queue() {
        let boss = CSPWorkerPool::new(None, Some(2), fn_test);
        boss.send_data("data 1");
        let results = boss.finish();
        assert_eq!(results[0], Ok("Result for data 1".to_string()))
    }
}
