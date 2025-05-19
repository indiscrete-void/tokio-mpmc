use tokio_mpmc::Queue;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let queue = Queue::new(10);

    // Spawn multiple receiver tasks
    let num_receivers = 3;
    let mut receiver_tasks = Vec::new();

    for i in 0..num_receivers {
        let receiver_queue = queue.clone();
        let task = tokio::spawn(async move {
            let mut count = 0;
            tracing::info!("Receiver {} started.", i);
            while let Ok(Some(value)) = receiver_queue.receive().await {
                tracing::info!("Receiver {} received value: {}", i, value);
                count += 1;
            }
            tracing::info!("Receiver {} finished. count: {}", i, count);
        });
        receiver_tasks.push(task);
    }

    // Send values after receivers are ready
    for i in 0..10 {
        queue.send(i).await.unwrap();
    }

    // Wait for a moment to allow receivers to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Close the queue after sending
    queue.close().await;

    // Wait for all receiver tasks to complete
    for task in receiver_tasks {
        task.await.unwrap();
    }

    tracing::info!("All receivers finished.");
}
