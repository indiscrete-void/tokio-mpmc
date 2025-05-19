use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Queue is closed")]
    QueueClosed,
}
