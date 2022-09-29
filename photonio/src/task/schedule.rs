use super::Task;

pub(crate) trait Schedule {
    fn schedule(&self, task: Task);
}
