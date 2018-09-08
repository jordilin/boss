extern crate boss;

use boss::CSPStreamWorkerPool;
use boss::CSPWorkerPool;
use std::{thread, time};

use std::io;

fn process_data(msg: i32) -> Result<i32, ()> {
    let one_second = time::Duration::from_millis(1000);
    println!("sent {}", msg);
    thread::sleep(one_second);
    Ok(msg)
}

fn main() -> io::Result<()> {
    let boss = CSPWorkerPool::new(None, Some(4), process_data, true);
    for i in 0..10 {
        boss.send_data(i);
    }
    let res = boss.finish();
    for r in res {
        if let Ok(r) = r {
            println!("{}", r);
        }
    }

    let boss = CSPStreamWorkerPool::new(None, Some(4), process_data);
    let rv = boss.clone();
    thread::spawn(move || {
        for i in 0..10 {
            boss.send_data(i);
        }
        boss.finish();
    });
    for r in rv {
        println!("rx {:?}", r.unwrap());
    }
    Ok(())
}
