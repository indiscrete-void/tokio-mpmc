use crate::benchmark::QueueBenchmark;
use tokio_mpmc::Queue;

/// Benchmark implementation for tokio-mpmc queue
pub struct TokioMpmcQueue {
    queue: Queue<u32>,
}

impl Clone for TokioMpmcQueue {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

#[async_trait::async_trait]
impl QueueBenchmark for TokioMpmcQueue {
    type Message = u32;

    fn new(capacity: usize) -> Self {
        Self {
            queue: Queue::new(capacity),
        }
    }

    async fn send(&self, msg: Self::Message) -> anyhow::Result<()> {
        self.queue.send(msg).await.map_err(|e| e.into())
    }

    async fn receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        self.queue.receive().await.map_err(|e| e.into())
    }

    async fn close(&self) {
        self.queue.close().await;
    }
}
