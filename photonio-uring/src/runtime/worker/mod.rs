use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    io::{Error, ErrorKind, Result},
    sync::mpsc,
    thread,
};

use io_uring::squeue;
use log::trace;
use scoped_tls::scoped_thread_local;

use crate::task::{Schedule, Task};

mod op;
use op::{OpHandle, OpTable};

mod driver;
use driver::{Driver, Unpark};

pub(crate) mod syscall;

enum Message {
    Shutdown,
    Schedule(Task),
}

type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

struct Local {
    id: usize,
    rx: Receiver,
    driver: Driver,
    run_queue: VecDeque<Task>,
    event_interval: usize,
}

impl Local {
    fn new(id: usize, rx: Receiver, event_interval: usize) -> Result<Self> {
        let driver = Driver::new()?;
        Ok(Self {
            id,
            rx,
            driver,
            run_queue: VecDeque::new(),
            event_interval,
        })
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let mut num_tasks = self.poll()?;
            for m in self.rx.try_iter() {
                match m {
                    Message::Shutdown => return Ok(()),
                    Message::Schedule(task) => {
                        task.poll();
                        num_tasks += 1;
                    }
                }
            }
            trace!("worker {} polled {} tasks", self.id, num_tasks);
            if num_tasks > 0 {
                self.driver.tick()?;
            } else {
                self.driver.park()?;
            }
        }
    }

    fn poll(&mut self) -> Result<usize> {
        let mut num_tasks = 0;
        while let Some(task) = self.run_queue.pop_front() {
            task.poll();
            num_tasks += 1;
            if num_tasks >= self.event_interval {
                break;
            }
        }
        Ok(num_tasks)
    }

    fn unpark(&self) -> Unpark {
        self.driver.unpark()
    }

    fn schedule(&mut self, task: Task) {
        self.run_queue.push_back(task);
    }
}

pub(super) struct Worker {
    tx: Sender,
    unpark: Unpark,
}

impl Worker {
    pub(super) fn spawn(
        id: usize,
        thread_name: String,
        thread_stack_size: usize,
        event_interval: usize,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let local = Local::new(id, rx, event_interval)?;
        let unpark = local.unpark();
        trace!("spawn thread {}", thread_name);
        thread::Builder::new()
            .name(thread_name)
            .stack_size(thread_stack_size)
            .spawn(move || enter(local))?;
        Ok(Self { tx, unpark })
    }

    pub(super) fn schedule(&self, task: Task) -> Result<()> {
        self.tx
            .send(Message::Schedule(task))
            .map_err(|_| Error::new(ErrorKind::Other, "send message to worker failed"))?;
        self.unpark.unpark()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.tx.send(Message::Shutdown).unwrap();
    }
}

#[derive(Clone)]
pub(super) struct Shared;

impl Shared {
    pub(super) fn new() -> Self {
        Self
    }
}

impl Schedule for Shared {
    fn schedule(&self, task: Task) {
        CURRENT.with(|context| {
            let local = unsafe { &mut *context.get() };
            local.schedule(task);
        })
    }
}

scoped_thread_local!(static CURRENT: UnsafeCell<Local>);

fn enter(local: Local) -> Result<()> {
    let context = UnsafeCell::new(local);
    CURRENT.set(&context, || {
        let local = unsafe { &mut *context.get() };
        local.run()
    })
}

fn submit(op: squeue::Entry) -> Result<OpHandle> {
    CURRENT.with(|context| {
        let local = unsafe { &mut *context.get() };
        local.driver.schedule(op)
    })
}
