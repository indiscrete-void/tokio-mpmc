use std::{hint::black_box, io::Write, time::Duration};
use tokio::task::JoinHandle;

pub mod flume_benchmark;
pub mod tokio_mpmc_benchmark;
pub mod tokio_mpsc_benchmark;

/// Define common behavior for queue benchmarking
#[async_trait::async_trait]
pub trait QueueBenchmark: Clone + Send + Sync + 'static {
    /// Message type in the queue
    type Message: Send + ToString + 'static + From<u32> + Into<u32>;

    /// Create a new queue instance
    fn new(capacity: usize) -> Self;

    /// Send a message to the queue
    async fn send(&self, msg: Self::Message) -> anyhow::Result<()>;

    /// Receive a message from the queue
    async fn receive(&mut self) -> anyhow::Result<Option<Self::Message>>;

    /// Close the queue
    async fn close(&self);
}

/// Define benchmark configuration parameters
pub struct BenchmarkConfig {
    pub queue_size: usize,
    pub num_producers: usize,
    pub num_consumers: usize,
    pub sample_size: usize,
    pub measurement_time: Duration,
    pub warm_up_time: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            queue_size: 10_000,
            num_producers: 4,
            num_consumers: 4,
            sample_size: 10,
            measurement_time: Duration::from_secs(15),
            warm_up_time: Duration::from_secs(2),
        }
    }
}

/// Run non-IO benchmark test
pub async fn run_non_io_benchmark<Q: QueueBenchmark>(config: &BenchmarkConfig) {
    let queue = Q::new(config.queue_size);
    let mut producer_handles = vec![];
    let mut consumer_handles = vec![];

    // Producer tasks
    for i in 0..config.num_producers {
        let queue = queue.clone();
        let messages_per_producer = config.queue_size / config.num_producers;
        let handle = tokio::spawn(async move {
            let start = i * messages_per_producer;
            let end = start + messages_per_producer;
            for msg in start..end {
                if queue
                    .send(black_box(Q::Message::from(msg.try_into().unwrap())))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Consumer tasks
    for _ in 0..config.num_consumers {
        let mut queue = queue.clone();
        let messages_per_consumer = config.queue_size / config.num_consumers;
        let handle = tokio::spawn(async move {
            let mut received = 0;
            while received < messages_per_consumer {
                match queue.receive().await {
                    Ok(Some(_)) => received += 1,
                    _ => break,
                }
            }
        });
        consumer_handles.push(handle);
    }

    wait_for_tasks(producer_handles, consumer_handles).await;
    queue.close().await;
}

/// Run IO benchmark test
pub async fn run_io_benchmark<Q: QueueBenchmark>(config: &BenchmarkConfig) {
    let queue = Q::new(config.queue_size);
    let mut producer_handles = vec![];
    let mut consumer_handles = vec![];

    // Producer tasks
    for i in 0..config.num_producers {
        let queue = queue.clone();
        let messages_per_producer = config.queue_size / config.num_producers;
        let handle = tokio::spawn(async move {
            let start = i * messages_per_producer;
            let end = start + messages_per_producer;
            for msg in start..end {
                if queue
                    .send(black_box(Q::Message::from(msg.try_into().unwrap())))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Consumer tasks (with IO operations)
    for i in 0..config.num_consumers {
        let mut queue = queue.clone();
        let messages_per_consumer = config.queue_size / config.num_consumers;
        let handle = tokio::spawn(async move {
            let mut received = 0;
            while received < messages_per_consumer {
                match queue.receive().await {
                    Ok(Some(msg)) => {
                        received += 1;
                        let _ = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(format!("bench_output_{}.txt", i))
                            .and_then(|mut file| file.write_all(msg.to_string().as_bytes()));
                    }
                    _ => break,
                }
            }
        });
        consumer_handles.push(handle);
    }

    wait_for_tasks(producer_handles, consumer_handles).await;

    // Clean up benchmark output files
    for i in 0..config.num_consumers {
        let _ = std::fs::remove_file(format!("bench_output_{}.txt", i));
    }

    queue.close().await;
}

/// Wait for all tasks to complete
async fn wait_for_tasks(
    producer_handles: Vec<JoinHandle<()>>,
    consumer_handles: Vec<JoinHandle<()>>,
) {
    for handle in producer_handles {
        let _ = handle.await;
    }
    for handle in consumer_handles {
        let _ = handle.await;
    }
}
