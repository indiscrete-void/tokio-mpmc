use crate::benchmark::QueueBenchmark;
use tokio::sync::mpsc;

/// Benchmark implementation for tokio-mpsc queue
pub struct TokioMpscQueue {
    sender: mpsc::Sender<u32>,
    receiver: mpsc::Receiver<u32>,
}

impl Clone for TokioMpscQueue {
    fn clone(&self) -> Self {
        let (sender, receiver) = mpsc::channel(self.sender.capacity());
        Self { sender, receiver }
    }
}

#[async_trait::async_trait]
impl QueueBenchmark for TokioMpscQueue {
    type Message = u32;

    fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self { sender, receiver }
    }

    async fn send(&self, msg: Self::Message) -> anyhow::Result<()> {
        self.sender.send(msg).await.map_err(|e| e.into())
    }

    async fn receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        Ok(self.receiver.recv().await)
    }

    async fn close(&self) {
        // mpsc queue automatically closes when all senders are dropped
        let _ = &self.sender;
    }
}
