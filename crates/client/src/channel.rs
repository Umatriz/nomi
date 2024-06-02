use std::ops::{Deref, DerefMut};

use tokio::sync::mpsc::{Receiver, Sender};

pub struct Channel<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        Self { tx, rx }
    }

    pub fn clone_tx(&self) -> Sender<T> {
        self.tx.clone()
    }
}

impl<T> Deref for Channel<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<T> DerefMut for Channel<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}
