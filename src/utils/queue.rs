use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub struct AsyncQueue<T> {
    data: RefCell<VecDeque<T>>,
    waiter: Cell<Option<Waker>>,
}

impl<T> Default for AsyncQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsyncQueue<T> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            waiter: Default::default(),
        }
    }

    pub fn push(&self, t: T) {
        self.data.borrow_mut().push_back(t);
        if let Some(waiter) = self.waiter.take() {
            waiter.wake();
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        self.data.borrow_mut().pop_front()
    }

    pub fn pop<'a>(&'a self) -> AsyncQueuePop<'a, T> {
        AsyncQueuePop { queue: self }
    }

    pub fn clear(&self) {
        mem::take(&mut *self.data.borrow_mut());
        self.waiter.take();
    }
}

pub struct AsyncQueuePop<'a, T> {
    queue: &'a AsyncQueue<T>,
}

impl<'a, T> Future for AsyncQueuePop<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(t) = self.queue.try_pop() {
            Poll::Ready(t)
        } else {
            self.queue.waiter.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}
