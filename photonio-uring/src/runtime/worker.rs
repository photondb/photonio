use std::{cell::RefCell, collections::VecDeque, future::Future, io::Result, sync::Mutex, thread};

use futures::channel::mpsc;
use io_uring::squeue;
use log::trace;
use scoped_tls::scoped_thread_local;

use super::{
    driver::{Driver, Op, Unpark},
    Shared,
};
use crate::task::{JoinHandle, Schedule, Task};

enum Message {
    Shutdown,
    Schedule(Task),
}

type Sender = mpsc::UnboundedSender<Message>;
type Receiver = mpsc::UnboundedReceiver<Message>;

struct Local {
    id: usize,
    shared: Shared,
    rx: RefCell<Receiver>,
    driver: RefCell<Driver>,
    run_queue: RefCell<VecDeque<Task>>,
    event_interval: usize,
}

impl Local {
    fn new(
        id: usize,
        rx: Receiver,
        unpark: Unpark,
        shared: Shared,
        event_interval: usize,
    ) -> Result<Self> {
        let driver = Driver::new(unpark)?;
        Ok(Self {
            id,
            shared,
            rx: RefCell::new(rx),
            driver: RefCell::new(driver),
            run_queue: RefCell::new(VecDeque::new()),
            event_interval,
        })
    }

    fn run(&self) -> Result<()> {
        let mut rx = self.rx.borrow_mut();
        loop {
            let mut num_tasks = self.poll()?;
            while let Ok(Some(msg)) = rx.try_next() {
                match msg {
                    Message::Shutdown => {
                        trace!("worker {} is shut down", self.id);
                        return Ok(());
                    }
                    Message::Schedule(task) => {
                        task.poll();
                        num_tasks += 1;
                    }
                }
            }
            trace!("worker {} polled {} tasks", self.id, num_tasks);
            {
                let mut driver = self.driver.borrow_mut();
                if num_tasks > 0 {
                    driver.tick()?;
                } else {
                    driver.park()?;
                }
            }
        }
    }

    fn poll(&self) -> Result<usize> {
        let mut num_tasks = 0;
        while num_tasks < self.event_interval {
            if let Some(task) = self.next_task() {
                task.poll();
                num_tasks += 1;
            } else {
                break;
            }
        }
        Ok(num_tasks)
    }

    fn next_task(&self) -> Option<Task> {
        let mut run_queue = self.run_queue.borrow_mut();
        run_queue.pop_front()
    }
}

pub(super) struct Worker {
    id: usize,
    tx: Sender,
    rx: Mutex<Option<Receiver>>,
    unpark: Unpark,
}

impl Worker {
    pub(super) fn new(id: usize) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded();
        let unpark = Unpark::new()?;
        Ok(Self {
            id,
            tx,
            rx: Mutex::new(Some(rx)),
            unpark,
        })
    }

    pub(super) fn launch(
        &self,
        shared: Shared,
        stack_size: usize,
        event_interval: usize,
    ) -> Result<()> {
        let rx = self.rx.lock().unwrap().take().unwrap();
        let local = Local::new(self.id, rx, self.unpark.clone(), shared, event_interval)?;
        let thread_name = format!("photonio-worker/{}", self.id);
        trace!("launch {}", thread_name);
        thread::Builder::new()
            .name(thread_name)
            .stack_size(stack_size)
            .spawn(move || enter(local))?;
        Ok(())
    }

    pub(super) fn schedule<F>(&self, id: u64, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (task, handle) = Task::new(id, future, Scheduler);
        self.tx.unbounded_send(Message::Schedule(task)).unwrap();
        self.unpark.unpark().unwrap();
        handle
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.tx.unbounded_send(Message::Shutdown).unwrap();
    }
}

scoped_thread_local!(static CURRENT: Local);

fn enter(local: Local) -> Result<()> {
    CURRENT.set(&local, || local.run())
}

/// Spawns a task onto the current runtime.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|local| local.shared.schedule(future))
}

pub(super) fn submit(op: squeue::Entry) -> Result<Op> {
    CURRENT.with(|local| {
        let mut driver = local.driver.borrow_mut();
        unsafe { driver.add(op) }
    })
}

struct Scheduler;

impl Schedule for Scheduler {
    fn schedule(&self, task: Task) {
        CURRENT.with(|local| {
            let mut run_queue = local.run_queue.borrow_mut();
            run_queue.push_back(task);
        })
    }
}
