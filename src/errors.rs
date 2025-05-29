use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel is full")]
    ChannelFull,
    #[error("Channel is closed")]
    ChannelClosed,
}

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Queue is closed")]
    QueueClosed,
}
