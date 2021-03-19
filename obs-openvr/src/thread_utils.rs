use std::{
    thread::JoinHandle,
};

pub struct JoinOnDrop<T>(Option<JoinHandle<T>>);

impl<T> From<JoinHandle<T>> for JoinOnDrop<T> {
    fn from(handle: JoinHandle<T>) -> Self {
        JoinOnDrop(Some(handle))
    }
}

impl<T> Drop for JoinOnDrop<T> {
    fn drop(&mut self) {
        if let Some(Err(e)) = self.0.take().map(JoinHandle::join) {
            error!("Failed joining thread upon drop: {:?}", &e);
        }
    }
}
