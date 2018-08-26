use crossbeam_channel::{Receiver, Sender};
use std::ops::Deref;

pub enum Work<T> {
    Data(T),
    Quit,
}

pub struct Rx<T>(pub Receiver<T>);
pub struct Tx<R>(pub Sender<R>);

impl<T> Deref for Rx<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Receiver<T> {
        &self.0
    }
}
