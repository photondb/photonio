pub trait Schedule {
    fn schedule(&self, task: Task);

    fn yield_now(&self, task: Task);
}
