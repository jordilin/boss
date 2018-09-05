use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use num_cpus;
use std::thread;

use data::{Rx, Tx, Work};

struct Worker<T, R> {
    rx: Rx<Work<T>>,
    tx: Tx<R>,
}

impl<T, R> Worker<T, R> {
    fn new(rx: Rx<Work<T>>, tx: Tx<R>) -> Self {
        Worker { rx, tx }
    }

    fn run<F>(&self, func: F)
    where
        F: Fn(T) -> R,
    {
        loop {
            match self.rx.recv() {
                Some(Work::Data(d)) => self.tx.0.send(func(d)),
                Some(Work::Quit) => {
                    break;
                }
                _ => break,
            }
        }
    }
}

#[derive(Clone)]
pub struct CSPStreamWorkerPool<T, R> {
    tx: Sender<Work<T>>,
    rx: Receiver<R>,
    num_workers: usize,
}

impl<T, R> CSPStreamWorkerPool<T, R> {
    pub fn new<F>(
        num_threads: Option<usize>,
        queue_size: Option<usize>,
        func: F,
    ) -> CSPStreamWorkerPool<T, R>
    where
        F: Fn(T) -> R + Send + Copy + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = match queue_size {
            Some(capacity) => bounded(capacity),
            None => unbounded(),
        };
        let num_workers = match num_threads {
            Some(val) => val,
            None => num_cpus::get(),
        };
        let (tx2, rx2) = unbounded();
        for _ in 0..num_workers {
            let rx = rx.clone();
            let tx2 = tx2.clone();
            thread::spawn(move || Worker::new(Rx(rx), Tx(tx2)).run(func));
        }
        CSPStreamWorkerPool {
            tx: tx,
            rx: rx2,
            num_workers: num_workers,
        }
    }

    pub fn send_data(&self, d: T) {
        self.tx.send(Work::Data(d));
    }

    pub fn finish(&self) {
        for _ in 0..self.num_workers {
            self.tx.send(Work::Quit);
        }
    }
}

impl<T, R> Iterator for CSPStreamWorkerPool<T, R> {
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        return self.rx.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fn_test(data: u32) -> u32 {
        data
    }

    #[test]
    fn streamer_workerpool() {
        let boss = CSPStreamWorkerPool::new(None, None, fn_test);
        boss.send_data(1);
        boss.send_data(2);
        boss.send_data(3);
        boss.finish();
        let mut count = 1;
        for r in boss {
            assert_eq!(count, r);
            count += 1;
        }
    }
}
