extern crate crossbeam_channel;
extern crate num_cpus;

pub mod block;
mod data;
pub mod stream;

pub use block::CSPWorkerPool;
pub use stream::CSPStreamWorkerPool;
