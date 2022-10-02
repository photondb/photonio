use std::{cell::RefCell, collections::VecDeque, io::Result};

use io_uring::squeue;
use scoped_tls::scoped_thread_local;

use super::{driver::Driver, op::OpHandle};
use crate::task::{Schedule, Task};

struct Local {
    id: usize,
    event_interval: usize,
    shared: Shared,
    driver: Driver,
    run_queue: VecDeque<Task>,
}

impl Local {
    fn new(id: usize, event_interval: usize) -> Result<Self> {
        let driver = Driver::new()?;
        Ok(Self {
            id,
            event_interval,
            shared: Shared,
            driver,
            run_queue: VecDeque::new(),
        })
    }

    fn schedule(&mut self, task: Task) {
        self.run_queue.push_back(task);
    }
}

#[derive(Clone)]
struct Shared;

impl Schedule for Shared {
    fn schedule(&self, task: Task) {
        CURRENT.with(|local| {
            let mut local = local.borrow_mut();
            local.schedule(task);
        })
    }
}

scoped_thread_local!(static CURRENT: RefCell<Local>);

pub(super) fn run(id: usize, event_interval: usize) -> Result<()> {
    let local = Local::new(id, event_interval)?;
    let context = RefCell::new(local);
    CURRENT.set(&context, || Ok(()))
}

pub(super) fn submit(sqe: squeue::Entry) -> Result<OpHandle> {
    CURRENT.with(|context| {
        let mut local = context.borrow_mut();
        local.driver.push(sqe)
    })
}
