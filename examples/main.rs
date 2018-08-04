#![feature(extern_prelude)]

use std::{thread, time};

use std::io;

fn process_data(msg: i32) {
    let one_second = time::Duration::from_millis(1000);
    thread::sleep(one_second);
    println!("{}", msg);
}

fn main() -> io::Result<()> {
    let mut boss = boss::create_bounded_workers(4, process_data);
    for i in 0..10 {
        boss.send_data(i);
        println!("sent {}", i);
    }
    boss.finish();
    Ok(())
}
