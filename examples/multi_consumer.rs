use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create a channel with buffer size 100
    let (tx, rx) = tokio_mpmc::channel(100);
    let rx = Arc::new(rx);

    // Number of consumer tasks
    let num_consumers = 4;
    let barrier = Arc::new(Barrier::new(num_consumers + 1)); // +1 for the main thread

    // Spawn consumer tasks
    let mut consumers = JoinSet::new();
    for consumer_id in 0..num_consumers {
        let rx = rx.clone();
        let barrier = barrier.clone();

        consumers.spawn(async move {
            let mut count = 0;
            let consumer_name = format!("consumer-{consumer_id}");

            // Wait for all consumers to be ready
            barrier.wait().await;

            loop {
                match rx.recv().await {
                    Ok(Some(value)) => {
                        // Simulate some processing time
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        tracing::info!("{} processed value: {}", consumer_name, value);
                        count += 1;
                    }
                    Ok(None) => {
                        tracing::info!("{}: Channel closed, exiting", consumer_name);
                        break;
                    }
                    Err(e) => {
                        tracing::error!("{}: Error receiving value: {:?}", consumer_name, e);
                        break;
                    }
                }
            }
            (consumer_name, count)
        });
    }

    // Wait for all consumers to be ready before starting to send messages
    barrier.wait().await;

    // Spawn producer task
    let producer = tokio::spawn(send_task(tx, 10));

    // Wait for producer to finish
    producer.await.unwrap();

    // Collect results from consumers
    let mut total_processed = 0;
    while let Some(Ok((name, count))) = consumers.join_next().await {
        tracing::info!("{} processed {} messages", name, count);
        total_processed += count;
    }

    tracing::info!("All done! Total messages processed: {}", total_processed);
}

async fn send_task(tx: tokio_mpmc::Sender<i32>, count: usize) {
    let mut tasks = JoinSet::new();

    // Spawn multiple producer tasks
    for i in 0..10 {
        let tx = tx.clone();
        let start = i * (count / 10);
        let end = if i == 9 {
            count
        } else {
            (i + 1) * (count / 10)
        };

        tasks.spawn(async move {
            for num in start..end {
                if let Err(e) = tx.send(num as i32).await {
                    tracing::error!("Error sending value: {}", e);
                    break;
                }
            }
            end - start
        });
    }

    // Wait for all producers to finish
    let mut total_sent = 0;
    while let Some(result) = tasks.join_next().await {
        total_sent += result.unwrap();
    }

    // Close the channel after all messages are sent
    drop(tx);

    tracing::info!("Producer finished. Total messages sent: {}", total_sent);
}
