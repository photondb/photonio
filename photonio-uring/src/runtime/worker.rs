use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    io::Result,
    sync::{Arc, Mutex},
    thread,
};

use buddy_alloc::buddy_alloc::BuddyAlloc;
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
    buf_alloc: Arc<UringBufAllocator>,
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
        let mut driver = Driver::new(unpark)?;
        let mut io_buf_alloc = Arc::new(UringBufAllocator::new(1 << 20, 0));
        {
            let alloc = io_buf_alloc.borrow_mut();
            let buf = alloc.as_bytes_mut();
            let bufs = &[libc::iovec {
                iov_base: buf.as_mut_ptr().cast(),
                iov_len: buf.len(),
            }];
            driver.register_buf(bufs)?;
        }
        Ok(Self {
            id,
            shared,
            rx: RefCell::new(rx),
            driver: RefCell::new(driver),
            run_queue: RefCell::new(VecDeque::new()),
            buf_alloc: io_buf_alloc,
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

pub(crate) struct UringBufAllocator {
    data: std::ptr::NonNull<u8>,
    layout: std::alloc::Layout,
    size: usize,
    allocator: RefCell<BuddyAlloc>,
    buffer_id: u16,
}

pub struct UringBuf {
    allocator: Arc<UringBufAllocator>,
    data: std::ptr::NonNull<u8>,
    buffer_id: u16,
    size: usize,
}

impl UringBuf {
    #[inline]
    pub fn as_bytes(&mut self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.size) }
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.size) }
    }

    #[inline]
    pub fn uring_buffer_index(&self) -> u16 {
        self.buffer_id
    }
}

impl UringBufAllocator {
    pub(crate) fn new(io_memory_size: usize, buffer_id: u16) -> Self {
        let size = Self::ceil_to_block_hi_pos(io_memory_size, 4096);
        let layout =
            std::alloc::Layout::from_size_align(size, 4096).expect("invalid layout for allocator");
        let (data, allocator) = unsafe {
            let data =
                std::ptr::NonNull::new(std::alloc::alloc(layout)).expect("memory is exhausted");
            let allocator = RefCell::new(BuddyAlloc::new(buddy_alloc::BuddyAllocParam::new(
                data.as_ptr(),
                layout.size(),
                layout.align(),
            )));
            (data, allocator)
        };
        Self {
            data,
            layout,
            size,
            allocator,
            buffer_id,
        }
    }

    pub(crate) fn alloc_buffer(self: Arc<Self>, n: usize, align: usize) -> Option<UringBuf> {
        let size = Self::ceil_to_block_hi_pos(n, align);
        let mut alloacator = self.allocator.borrow_mut();
        std::ptr::NonNull::new(alloacator.malloc(size)).map(|data| UringBuf {
            allocator: self.clone(),
            data,
            buffer_id: self.buffer_id,
            size,
        })
    }

    #[allow(clippy::mut_from_ref)]
    // this only be used to construct iovec to register buf, the register buffer's
    // iovec no need mut in fact.
    pub(crate) fn as_bytes_mut(&self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.size) }
    }

    #[inline]
    fn ceil_to_block_hi_pos(pos: usize, align: usize) -> usize {
        ((pos + align - 1) / align) * align
    }
}

impl Drop for UringBufAllocator {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.data.as_ptr(), self.layout);
        }
    }
}

impl Drop for UringBuf {
    fn drop(&mut self) {
        let ptr = self.data;
        let mut alloc = self.allocator.allocator.borrow_mut();
        alloc.free(ptr.as_ptr() as *mut u8);
    }
}

unsafe impl Send for UringBufAllocator {}

unsafe impl Sync for UringBufAllocator {}

unsafe impl Send for UringBuf {}

unsafe impl Sync for UringBuf {}

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

/// Alloc io_buf from uring.
/// return None if unsupport or no engouh memory.
pub fn alloc_uring_buf(n_bytes: usize, align: usize) -> Option<UringBuf> {
    CURRENT.with(|local| {
        let alloc = local.buf_alloc.clone();
        alloc.alloc_buffer(n_bytes, align)
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
