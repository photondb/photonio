use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub struct JoinHandle<T> {
    _mark: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn is_finished(&self) -> bool {
        todo!()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
