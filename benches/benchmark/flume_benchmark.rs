use crate::benchmark::QueueBenchmark;

/// Benchmark implementation for flume queue
pub struct FlumeQueue {
    sender: flume::Sender<u32>,
    receiver: flume::Receiver<u32>,
}

impl Clone for FlumeQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

#[async_trait::async_trait]
impl QueueBenchmark for FlumeQueue {
    type Message = u32;

    fn new(capacity: usize) -> Self {
        let (sender, receiver) = flume::bounded(capacity);
        Self { sender, receiver }
    }

    async fn send(&self, msg: Self::Message) -> anyhow::Result<()> {
        self.sender.send_async(msg).await.map_err(|e| e.into())
    }

    async fn receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        match self.receiver.recv_async().await {
            Ok(msg) => Ok(Some(msg)),
            Err(_) => Ok(None),
        }
    }

    async fn close(&self) {
        // flume queue automatically closes when all senders are dropped
        let _ = &self.sender;
    }
}
