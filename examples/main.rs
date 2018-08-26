extern crate boss;
use std::{thread, time};

use boss::block::CSPWorkerPool;
use std::io;

fn process_data(msg: i32) -> Result<i32, ()> {
    let one_second = time::Duration::from_millis(1000);
    thread::sleep(one_second);
    Ok(msg)
}

fn main() -> io::Result<()> {
    let mut boss = CSPWorkerPool::new(None, Some(4), process_data);
    for i in 0..10 {
        boss.send_data(i);
        println!("sent {}", i);
    }
    let res = boss.finish();
    for r in res {
        if let Ok(r) = r {
            println!("{}", r);
        }
    }
    Ok(())
}
