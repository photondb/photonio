use std::{cell::RefCell, collections::VecDeque, io::Result};

use io_uring::squeue;
use scoped_tls::scoped_thread_local;

use super::{driver::Driver, op::OpHandle, Message, Receiver};
use crate::task::{Schedule, Task};

struct Local {
    _id: usize,
    _shared: Shared,
    event_interval: usize,
    rx: Receiver,
    driver: Driver,
    run_queue: VecDeque<Task>,
}

impl Local {
    fn new(id: usize, rx: Receiver, shared: Shared, event_interval: usize) -> Result<Self> {
        let driver = Driver::new()?;
        Ok(Self {
            _id: id,
            _shared: shared,
            event_interval,
            rx,
            driver,
            run_queue: VecDeque::new(),
        })
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let num_tasks = self.poll()?;
            if num_tasks > 0 {
                self.driver.tick()?;
            } else {
                self.driver.park()?;
            }
            for m in self.rx.try_iter() {
                match m {
                    Message::Shutdown => return Ok(()),
                    Message::Schedule(task) => self.run_queue.push_back(task),
                }
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

    fn schedule(&mut self, task: Task) {
        self.run_queue.push_back(task);
    }
}

#[derive(Clone)]
pub(crate) struct Shared;

impl Shared {
    pub fn new() -> Self {
        Self
    }
}

impl Schedule for Shared {
    fn schedule(&self, task: Task) {
        CURRENT.with(|local| {
            let mut local = local.borrow_mut();
            local.schedule(task);
        })
    }
}

scoped_thread_local!(static CURRENT: RefCell<Local>);

pub(super) fn run(id: usize, rx: Receiver, shared: Shared, event_interval: usize) -> Result<()> {
    let local = Local::new(id, rx, shared, event_interval)?;
    let context = RefCell::new(local);
    CURRENT.set(&context, || {
        let mut local = context.borrow_mut();
        local.run()
    })
}

pub(super) fn submit(sqe: squeue::Entry) -> Result<OpHandle> {
    CURRENT.with(|context| {
        let mut local = context.borrow_mut();
        local.driver.push(sqe)
    })
}
