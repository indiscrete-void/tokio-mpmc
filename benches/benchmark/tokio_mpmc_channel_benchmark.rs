use crate::benchmark::QueueBenchmark;

/// Benchmark implementation for tokio-mpmc channel.
pub struct TokioMpmcChannel {
    sender: tokio_mpmc::Sender<u32>,
    receiver: tokio_mpmc::Receiver<u32>,
}

impl Clone for TokioMpmcChannel {
    fn clone(&self) -> Self {
        let (sender, receiver) = tokio_mpmc::channel(self.sender.capacity());
        Self { sender, receiver }
    }
}

#[async_trait::async_trait]
impl QueueBenchmark for TokioMpmcChannel {
    type Message = u32;

    fn new(capacity: usize) -> Self {
        let (sender, receiver) = tokio_mpmc::channel(capacity);
        Self { sender, receiver }
    }

    async fn send(&self, msg: Self::Message) -> anyhow::Result<()> {
        self.sender.send(msg).await.map_err(|e| e.into())
    }

    async fn receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        self.receiver.recv().await.map_err(|e| e.into())
    }

    async fn close(&self) {
        // mpmc channel automatically closes when all senders are dropped
        let _ = &self.sender;
    }
}
